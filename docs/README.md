# Technical documentation

How the Truco app is built, how to work on it and how releases are made. For what the game is and how to play it, see the [main README](../README.md).

## Stack

- **Rust** (edition 2024). The toolchain is pinned in [`rust-toolchain.toml`](../rust-toolchain.toml).
- **[iced](https://iced.rs) 0.14** for the UI, with the `tokio` feature (bot move timer) and the `canvas` feature (table art and card pile).
- **[fastrand](https://crates.io/crates/fastrand)** for shuffling and the bots' randomness.
- **[png](https://crates.io/crates/png)**, optional, only with the `screenshots` feature.

## Project structure

```
src/
├── main.rs          # app entry point: builds the iced application
├── bot.rs           # opponent AI per difficulty (the heuristic also powers hints)
├── i18n.rs          # Portuguese (default) and English text
├── opponents.rs     # the four fictional opponents and their difficulty
├── game/
│   ├── card.rs      # suits, ranks, cards, deck, strength with the vira
│   ├── engine.rs    # match state machine: tricks, raises, scoring
│   └── variant.rs   # Paulista vs Mineiro: manilhas, stake values, special hand
└── ui/
    ├── mod.rs       # iced application: state, messages, update, subscription
    ├── menu.rs      # main menu (game mode, language)
    ├── opponents.rs # opponent selection screen
    ├── table.rs     # game screen
    ├── tutorial.rs  # step-by-step tutorial and quizzes
    ├── scene.rs     # canvas art: bar wall, round table, card pile, props
    ├── widgets.rs   # cards, panels, avatars and colors
    └── capture.rs   # screenshot tour (feature `screenshots`)
docs/
├── README.md        # this file
└── screenshots/     # images used by the main README
```

## Architecture

The code is split into three layers. Each one depends only on the layers below it:

```
ui  ──►  bot, i18n, opponents  ──►  game
```

### `game`: the rules engine

`game` knows nothing about the UI or the bots. It can be driven by tests, by the bot or by the screen in the same way.

- **`Card`**: rank plus suit. `Card::strength(vira)` gives the Paulista strength: 0–9 by rank, 10–13 for manilhas ordered ♦ < ♠ < ♥ < ♣.
- **`Variant`**: everything that differs between Truco Paulista and Truco Mineiro:
  - `strength` / `is_manilha`: the vira-based manilhas, or Mineiro's fixed 4♣ > 7♥ > A♠ > 7♦
  - `stake_value`: 1/3/6/9/12 or 2/4/8/10/12
  - `special_hand_score`: 11 (*mão de onze*) or 10 (*mão de dez*)
  - `uses_vira`
- **`Game`**: the match state machine. The UI reads its public fields (`hands`, `table`, `hand_plays`, `tricks`, `scores`, `stake`, `events`…) and changes it only through:
  - `actor()`: who must act now, if anyone
  - `is_legal(player, action)` / `raise_target(player)`
  - `apply(player, action)`: one of `Play(index)`, `Raise`, `Accept`, `Fold`
  - `next_hand()`: deals the next hand once the current one is over

The match moves through these phases (`Phase`):

| Phase | Who acts | What can happen next |
| --- | --- | --- |
| `ElevenDecision` | The player with 11 (Paulista) or 10 (Mineiro) points | `Accept` → `Playing`, with the hand worth the truco value. `Fold` → the opponent scores the base value → `HandOver` |
| `Playing` | `turn` | `Play` a card: when the second card lands, the round is resolved, and the hand may be decided → `HandOver`. `Raise` → `AwaitingResponse` |
| `AwaitingResponse` | The player who was raised | `Accept` → back to `Playing` at the new stake. `Raise` again → the other player must respond. `Fold` → the raiser scores the current stake → `HandOver` |
| `HandOver` | Nobody | `next_hand()` deals again. The hand starter alternates |
| `MatchOver` | Nobody | Someone reached 12 points |

- `Stake` is a level on the raise ladder (`Base`, `Truco`, `Six`, `Nine`, `Twelve`). Its point value always comes from the `Variant`.
- The player who raised last cannot raise again until the other player does (`last_raiser`).
- Raising is forbidden in the special hand and the iron hand (`eleven_hand`).
- `decide_hand` applies the tie rules: a tied first round goes to the next decisive round, a later tie goes to the winner of the first round, and three ties score nothing.
- Every state change also pushes an `Event`. The UI turns these into the history log, in the selected language.

### `bot`: the opponents

`bot::decide_as(difficulty, game, player, rng)` returns a `Decision` (an action plus a `Reason`):

| Difficulty | Opponent | Strategy |
| --- | --- | --- |
| Beginner | Tonho | Heuristic with ±0.35 confidence noise, a random card 50% of the time, never raises back or bluffs |
| Amateur | Dona Cida | Heuristic with ±0.15 noise, a random card 20% of the time, bluffs 3% of the time |
| Professional | Seu Chico | Textbook heuristic, bluffs 7% of the time |
| Master | Bigode | Monte Carlo search, described below |

- **Heuristic.** It estimates a confidence from the cards left in hand and the rounds already won or lost. For card play, it beats the opponent's card with the cheapest card that wins, ties when a tie is enough, otherwise discards its weakest card. When leading, it plays a medium card first, then a weak or strong one depending on the score.
- **Master.** For each candidate move it runs about 160 rollouts. Each rollout deals the opponent a random hand drawn only from cards the master has not seen (so it never cheats) and plays the hand out with the heuristic. It plays the card with the best win rate and decides truco by comparing expected points: `(2p − 1) × value` against the sure loss of folding.
- **Hints.** They call `bot::decide(game, Player::Human, None)`, the deterministic professional heuristic. Its `Reason` is explained to the player, in the selected language, by `i18n`.

### `i18n`: translations

- `Language` (PT-BR by default, or English) points to a static `Strings` table for fixed labels and the tutorial steps.
- Texts that depend on the game state (event log, prompts, hand value, opponent names) are methods on `Language`. They take the `Variant` and the opponent's name as parameters.
- To add a language, add a variant to `Language` and fill in a new `Strings` table, plus the `match` arms the compiler will then require.

### `ui`: the iced application

- **`App`** holds the screen (`Menu`, `Tutorial`, `Opponents`, `Game`), the language, the selected `Variant`, the current opponent, the `Game` and the hints toggle.
- **Bot timing.** `App::subscription` emits `BotTick` every 900 ms while it is the bot's turn, so the player can follow each move.
- **Layering.** Every screen draws on top of a canvas wall (`scene::Wall`). The game screen stacks the widgets (bot cards, prompt, buttons, player's hand) on top of `scene::Table`.
- **`scene::Table`** draws the round table, the deck with the vira, the props, and the **pile**: every card in `Game::hand_plays`, tossed toward its owner with a deterministic random twist.
  - Pile cards are always drawn slightly rotated. iced turns rotated canvas text into geometry, which keeps the layers stacked correctly; unrotated canvas text would always render on top of everything.

## Development

```sh
cargo run                                   # run the app (debug)
cargo run --release                         # run the app (optimized)
cargo test                                  # unit tests
cargo clippy --all-targets --all-features   # lint (CI expects no warnings)
cargo fmt --all -- --check                  # formatting
```

The tests cover:
- card strength and the manilha rules of both variants
- the tie rules and raise and fold scoring
- the special hand
- hundreds of full random and bot-vs-bot matches in both variants
- a strength ladder showing that harder opponents win more often

Formatting uses hard tabs (see [`.rustfmt.toml`](../.rustfmt.toml)). The other options in that file are nightly-only, and stable `rustfmt` ignores them with a warning.

### Screenshots

The images in [`screenshots/`](screenshots) are taken by the app itself. With the `screenshots` feature, it walks through a scripted tour (menu, opponents, tutorial, Paulista, Mineiro and English games), captures each scene with `iced::window::screenshot`, writes the PNGs and quits:

```sh
cargo run --features screenshots
```

The scenes are listed in `src/ui/capture.rs`. The game scenes come from deterministic seeds, so the images are reproducible.

## CI and releases

- **CI** ([`.github/workflows/CI.yml`](../.github/workflows/CI.yml)) builds and tests every pull request.
- **Renovate** keeps dependencies and the Rust toolchain up to date, using `chore(deps):` commits.
- **Releases** are automated with [release-plz](https://release-plz.dev) ([`release-plz.toml`](../release-plz.toml), [`.github/workflows/release-plz.yml`](../.github/workflows/release-plz.yml)):
  1. Every push to `main` opens or updates a `chore: release vX.Y.Z` PR. That PR bumps the version in `Cargo.toml` and `Cargo.lock` and adds the `CHANGELOG.md` section, generated from conventional commits (`feat`, `fix`, `docs`, `perf`, `refactor`…).
  2. Merging that PR creates the `vX.Y.Z` tag and the GitHub release.
  3. Nothing is published to crates.io (`git_only = true`); versions come from git tags.
  4. Right after the release is created, [`.github/workflows/build-installers.yml`](../.github/workflows/build-installers.yml) builds the installers with [cargo-packager](https://github.com/crabnebula-dev/cargo-packager) and attaches them to the release: `Truco-Windows-Setup.exe` (NSIS), `Truco-macOS.dmg` (universal, ad-hoc signed), `Truco-Linux-x86_64.AppImage` and `Truco-Linux-amd64.deb`. The names never change, so the landing page links to `releases/latest/download/<file>`. The same workflow runs as a check on pull requests that touch packaging, and can be started by hand from the Actions tab with a tag.

  The workflow uses the `RELEASE_PLZ_TOKEN` secret (a fine-grained token limited to this repository, with read and write access to *Contents* and *Pull requests*) so that release PRs also trigger CI.

## Packaging and icons

- The packaging config lives in `[package.metadata.packager]` in [`Cargo.toml`](../Cargo.toml). To build an installer for your own system locally:

  ```sh
  cargo install cargo-packager --locked
  cargo build --release && cargo packager --release
  ```

- The app icon is in [`assets/`](../assets): `icon-1024.png` is the source, the smaller PNGs, `icon.ico` (Windows) and `icon.icns` (macOS) are derived from it.
- [`build.rs`](../build.rs) embeds `icon.ico` into the Windows executable; `src/main.rs` sets `icon-64.png` as the window icon and hides the console window in Windows release builds.

