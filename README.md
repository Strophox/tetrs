<div align="center"><img width="440" src="Gallery/tetrs_logo.png" /></div>


<div align="center" style="text-align: center; width: 100%">
<h1>Tetromino Game Engine + Terminal Application</h1>
</div>

*This repo hosts*
- `tetrs_terminal`, a simple, moderately polished cross-platform TUI implementation of the typical game, and
- `tetrs_engine`, a tetromino game engine implementing an interface capable of handling modern mechanics.

---

**(Author's Note : Due to irl circumstances I cannot continue development right now - issues may be worked on at a later time)**

---


## How to run
- [Download a release](https://github.com/Strophox/tetrs/releases) for your platform if available.
- Run the application in your favourite terminal

> Or compile it yourself:
> - Have [Rust](https://www.rust-lang.org/) (1.80.0+) installed.
> - Download / `git clone` this repository.
> - Navigate to `tetrs_terminal/` and `cargo run`.

> [!IMPORTANT]
> Use a terminal like [kitty](<https://sw.kovidgoyal.net/kitty/>) (or any terminal with [support for progressive keyboard enhancement](https://docs.rs/crossterm/latest/crossterm/event/struct.PushKeyboardEnhancementFlags.html)) for smoother gameplay experience.
> **Note that otherwise DAS/ARR/Soft drop speed will be determined by Keyboard/OS/terminal emulator settings.** 
> 
> > <details>
> > 
> > <summary> Explanation. </summary>
> > 
> > Terminals do not usually send "key released" signals, which is a problem for mechanics such as "press left to move left repeatedly **until key is released**".
> > [Crossterm](https://docs.rs/crossterm/latest/crossterm/) automatically detects ['kitty-protocol'-compatible terminals]([https://docs.rs/crossterm/latest/crossterm/event/struct.PushKeyboardEnhancementFlags.html](https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement)) where this issue is solved, allowing for smooth, configurable gameplay controls.
> >
> > (\*This also affects holding Soft Drop locking pieces on ground instantly, as opposed to only upon press down -- for ergonomics this is explicitly mitigated by the 'No soft drop lock' configuration.)*
> > 
> > </details>


## Gallery

*Classic game experience with different gamemodes:*

![Tetrs demo screenshot](Gallery/Screenshots/tetrs_screenshot-game.png)


*Smooth rendering on all platforms, configurable controls and more:*

![Tetrs demo GIF](Gallery/Gifs/tetrs_rec-main.gif)


**ASCII graphics available:**

<details>

<summary>ASCII demo GIF</summary>

![Tetrs ASCII demo GIF](Gallery/Gifs/tetrs_rec-ascii.gif)

</details>


**Retro 'Electronika 60' graphics available:**

<details>

<summary>Electronika 60 demo PNG</summary>

![Electronika 60 demo PNG](Gallery/Screenshots/tetrs_screenshot-electronika-60.png)

*\*For display like in the screenshot set your terminal text color to green and font to a Unicode-compatible one (e.g. `DejaVu Sans Mono` works)*

</details>

> [!TIP]
> Play **Puzzle Mode** with its 24 stages to try the special ['ocular' rotation system](#ocular-rotation-system) *(feat. T-spin Triple)*!
> 
> <details>
> 
> <summary> Puzzle Mode demo GIF </summary>
> 
> ![Tetrs Puzzle Mode demo GIF](Gallery/Gifs/tetrs_rec-puzzle.gif)
> 
> </details>


# Features of the Application

### Gameplay
- Familiar stacker experience of moving, rotating, soft-/hard-dropping and holding *tetrominos* and clearing completed rows.
- Colorful pieces.
- Next piece preview.
- Ghost piece.
- Animations: Hard drop, Line clears and Piece locking.
- Game stats: Level, Score, Lines, Time, Pieces generated.

For more technical details see [Features of the Tetrs Engine](#features-of-the-tetrs-engine).

### Gamemodes
- **40-Lines**: Clear 40-Lines as quickly as possible.
- **Marathon**: Reach the highest speed level (with the highest score possible).
- **Master**: Clear 300 lines starting *at* the highest speed level.
- **Cheese**: Eat yourself through 32 lines with random holes (with as few pieces as possible).
- **Puzzle**: Advance through all 24 puzzle stages using perfect clears (and up to 5 attempts), enabled by piece acrobatics of the 'ocular' rotation system.
- **Custom**: Change start level, toggle level increment, set game limit *(Time, Score, Pieces, Lines, Level, or No limit)*.
  
### Settings
- Look of the game:
  - Graphics (Unicode, ASCII, 'Electronika 60').
  - Coloring (RGB Colors; 16 Colors (should work on all consoles), Monochrome).
  - Adjustable render rate and toggleable FPS counter.
- Play of the game:
  - Change controls.
  <details>
  
  <summary> Default Game Controls </summary>
  
  | Key | Action |
  | -: | :-: |
  | `←` | Move left |
  | `→` | Move right |
  | `A` | Rotate left |
  | `D` | Rotate right |
  | (not set) | Rotate around (180°) |
  | `↓` | Soft drop |
  | `↑` | Hard drop |
  | (not set) | Sonic drop |
  | `Esc` | Pause game |
  | `Ctrl`+`D` | Forfeit game |
  | `Ctrl`+`C` | Exit program |
  
  </details>
  
  - Configure game.
    - Rotation system  (Ocular, Classic, Super),
    - Piece generator (History, Uniform, Bag, Total-Relative),
    - Preview count (0 - 8),
    - DAS, ARR, hard drop delay, line clear delay, appearance delay,
    - soft drop factor, ground time max,
  - *Advanced*, No soft drop lock (Enables soft drop not instantly locking pieces on ground even if keyboard enhancements are off, for better experience on typical consoles (soft drops for piece spins)).
- **Keep Savefile**: By default this program won't store anything and just let you play the game. If you **do** want `tetrs_terminal` to restore your settings and past games in the future then make sure this is set to **"On"**!
  
### Scoreboard
- History of games played in the current session (or in the past, if "keep save file" is toggled on).
- *(\*Games where 0 lines have been cleared are auto-deleted on exit.)*

> [!NOTE]
> If "keep save file for tetrs" is toggled ON then your settings and games will be stored in `.tetrs_terminal.json` under a directory that tries to follow OS conventions [[1](https://softwareengineering.stackexchange.com/questions/3956/best-way-to-save-application-settings), [2](https://softwareengineering.stackexchange.com/questions/299869/where-is-the-appropriate-place-to-put-application-configuration-files-for-each-p)]:
> | | Windows | Linux | macOS | other |
> | -: | - | - | - | - |
> | location | `%APPDATA%` | `~/.config/` | `~/Library/Application Support/` | (home directory) |
> 
> (If this fails it tries to store it locally, `./`.)


# Features of the Tetrs Engine

The frontend application is proof-of-concept;
Ultimately the tetrs engine tries to be modular and shifts the responsibility of detecting player input and chosen time of updates to the client.
Basic interaction with the engine could look like the following:

```rust
// Starting a game.
let game = tetrs_engine::Game::new(Gamemode::marathon());

// Application loop.
loop {
  // Updating the game with a new button state at a point in time.
  game.update(Some(new_button_state), update_time);
  // ...
  // Updating the game with *no* change in button state (since previous).
  game.update(None, update_time_2);

  // View game state
  let GameState { board, .. } = game.state();
  // (Render the board, etc..)
}
```

<details>

<summary> Using the engine in a Rust project </summary>

Adding `tetrs_engine` as a [dependency from git](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) to a project:

```toml
[dependencies]
tetrs_engine = { git = "https://github.com/Strophox/tetrs.git" }
```

</details>

<details>

<summary> Game Configuration Aspects </summary>

- Gamemodes: Are encoded as a combination of *starting level* and *whether to increment level* and (one/several positive/negative) *limits*.
- Rotation Systems: *Ocular Rotation System*, *Classic Rotation System*, *Super Rotation System*. See [Ocular Rotation System](#ocular-rotation-system).
- Tetromino Generators: *Recency-based*, *Bag*, *Uniformly random*. Default is recency. See [Tetromino Generation](#tetromino-generation).
- Piece Preview (default 1)
- Delayed Auto Shift (default DAS = 167ms) *(\*Note: at very high levels [DAS and ARR equal lock delay - 1ms](https://www.reddit.com/r/Tetris/comments/cjkosd/tetris_effect_master_mode/).)*
- Auto Repeat Rate (default ARR = 33ms)
- Soft Drop Factor (default SDF = 15.0)
- Hard drop delay (default 0.1ms)
- Line clear delay (default 200ms)
- Appearance Delay (default ARE = 50ms)

Currently, drop delay and lock delay\* *(\*But not total ground time)* are a function of the current level:
- Drop delay (1000ms at lvl 1 to 0.833ms ("20G") at lvl 19, as per guideline)
- 'Timer' variant of Extended Placement Lockdown (step reset); The piece tries to lock every 500ms at lvl 19 to every 150ms at lvl 30, and any given piece may only touch ground for 2250ms in total. See [Piece Locking](#piece-locking).

All default values loosely based on the [Guideline](https://tetris.wiki/Tetris_Guideline).

</details>

<details>

<summary> Game State Aspects </summary>

- Time: Game time is held abstract as "time elapsed since game started" and is not directly tied to real-world timestamps.
- Game finish: The game knows if it finished, and if session was won or lost. Normal Game Over scenarios are:
  - Block out: new piece spawn location is occupied.
  - Lock out: a piece was completely locked above the skyline (row 21 and above).
  - Forfeit: player stopped the current.
- Event queue: All game events are kept in an internal queue that is simulated through (up to the provided timestamp in the `Game::update` call).
- Buttons pressed state: The game keeps an abstract state of which buttons are currently pressed.
- Board state: (Yes).
- Active piece: The active piece is stored as a (tetromino, orientation, position) tuple plus some locking data.
- Next Pieces: Are polled from the generator, kept in a queue and can be viewed.
- Pieces played so far: A counter for each locked piece by type is stored.
- Lines cleared: (Yes)<sup>2</sup>.
- (Speed) Level: Increases every 10 line clears and influences only drop/lock delay.
- Scoring: Line clears trigger a score bonus, which takes into account number of lines cleared, spins, combos, back-to-backs; See [Scoring](#scoring).
  
</details>


<details>

<summary> Game Feedback Aspects </summary>

The game provides some useful feedback events upon every `update`, usually used to correctly implement visual frontend effects:
- *Piece locked down*, *Lines cleared*, *Hard drop*, *Accolade* (score bonus info), *Message* (generic message, currently unused for base gamemodes)

</details>


# State of the Project

As much love and care went into building this project, it is of course not without its flaws;

- The README is not comprehensive:
  - Many small details of the `tetrs_engine` are not properly explained (e.g. the initial rotation mechanic, which allows spawning a piece immediately rotated if a rotation button was held).
- The engine itself might contain niche bugs as of this time. Concrete improvements include:
  - Better API documentation (`cargo doc --open`).
  - Simplification of code.
  - Proper commenting of implementation.
  - Refactor of complicated systems (e.g. locking).
  - Ideally: ensuring that `Game::update` is safe / actually panic-free.
- With regards to the terminal game experience, I'd like to argue the frontend is polished enough (much dedication went into making it nice for a 'proof-of-concept'). Regardless of whether this is actually the case, it is very lacking in aspects of code style, defects include:
  - The code for the menus is ad-hoc,
  - code duplication runs rampant,
  - no (or even worse, *wrong*) comments, and
  - possible panics may still be hiding around the corner.

A goal of mine would be to (at least partially) amend these problems, step-by-step.


# Project Highlights

While the [2009 Tetris Guideline](https://tetris.wiki/Tetris_Guideline) serves as good inspiration, I ended up doing a lot of amateur research into a variety of game details present in modern games online (thank you [Tetris Wiki](https://tetris.wiki/) and [HardDrop](https://harddrop.com/wiki)!) and also by getting some help from asking people. Thank you GrBtAce and KonSola5!

In the following I detail various interesting concepts I tackled on my way to bringing this project to life - I was essentially new to Tetris and couldn't remember playing it for more than a couple minutes (in the last decade), so I had to figure all this out from scratch!


## Tetromino Generation

[Tetromino generators are interesting](https://simon.lc/the-history-of-tetris-randomizers), and a core part of the game.

A trivial generator chooses tetrominos *uniformly at random*.
This already works decent for a playable game.

However, typically players tend to get very frustrated when they don't get "the pieces they need" for a while.
*(\*There's even a term for not getting an `I`-piece for an extended period of time: Drought.)*

In modern games, the *bag-based generation system* is ubiquitous;
The system simply takes all 7 pieces once, hands them out in random order, repeat.

It's quite nice knowing an `I` piece will come after 12 pieces, every time. One may even start strategizing and counting where one bag ends and the next starts.

It also takes a bit of the "fun" out of the randomness.

An alternative that seems to work well is *recency-based generation*:
Remember the last time each piece was generated and when choosing the next one randomly, do so weighted by when each one was last seen so it is more likely to choose a piece not played in a while.

This preserves "possibly complete randomness" in the sense that *any* piece may be generated at any time, while still mitigating droughts.

Unlike bag it possibly provides a more continuous "gut feeling" of what piece(s) might come next, where in bag the order upon refill really *is* completely random.


## Ocular Rotation System

> "[tetris has a great rotation system and is not flawed at all](https://www.youtube.com/watch?v=_qaEknA81Iw)"

— *said no one ever, not even [the creator of Tetris himself](https://youtu.be/6YhkkyXydNI?si=jbVwfNtfl5yFh9Gk&t=674).*

Considering the sheer size of the franchise and range of players coming from all sorts of niches and previous game version the official *Super Rotation System* ['gets its job done'](https://www.youtube.com/watch?v=dgt1kWq2_7c)™ - nevertheless it was *the* mechanic I wanted to redo even before starting this project.

<details>

<summary> Creating a Rotation System. </summary>

My personal gripes with SRS are:

- The system is not symmetric.
  - Symmetric pieces can look exactly the same in different rotation states, **[but have different behaviour](https://blog.battlefy.com/how-accidental-complexity-harms-the-tetris-community-93311461b2af)**.
  - Doing rotation, then Mirroring board and piece **≠** Mirroring board and piece, then Doing mirrored rotation.
- It's an [advanced system](https://harddrop.com/wiki/SRS) with things like different rotation points for different purposes, yet it re-uses the *exact same kicks* for 5 out of the 7 pieces, even though they have completely different symmetries.
- <sup>Not a hot take, but some rotations are just *weird* (to be chosen over other possibilities).</sup>
- Piece elevators.

Good general criteria for a rotation system I can think of would be:

1. Rotation must behave visually **symmetrically**;
    - equal-looking rotation states must behave the same,
    - and mirroring the board/pieces mirrors the rotation behaviour perfectly.
2. The kicks should be **intuitive**
    - the way pieces rotate should look 'feasible' to any given person;
    - e.g. the new position cannot be completely disjoint from previously.
3. Rotation should be fun! :D
    - but also any rotations should 'feel good' to a player.
    - (but don't overdo it - no teleporting pieces.)

The result of this was the *'Ocular' Rotation System*, which was made by... *looking* at each piece and orientation and drawing the 'best' position(s) for it to land in after rotating (following above points and gut feeling).

I present to you - the Ocular Rotation System Heatmap:

![Ocular Rotation System Heatmap](Gallery/ocular-rotation-system_16px.png)

*How to read it*:
This heatmap is created by considering each combination of (piece, orientation, rotate left or right).
By overlapping all the _new_ possible positions for a piece after rotation, one gets a compact visualization of where the piece will most likely land, going from brightest color (yellow, first position attempt) to darkest (purple, last attempt):

Here's a comparison with SRS - the Super Rotation System Heatmap:

![Super Rotation System Heatmap](Gallery/super-rotation-system_16px.png)

With SRS one starts to spot some rotational symmetries (you can always rotate back-and-forth between two positions), but I think it's overshadowed by all the asymmetrical kicks and very lenient (downwards *and upwards*) vertical kicks that contribute to SRS' unintuitiveness.

</details>

In the end I'm happy with how the custom rotation system turned out.
It vaguely follows the mantra "if the rotation looks like it could reasonably work visually, it should" (+ some added kicks for flexibility and fun :-), hence it's name, *Ocular* rotation system[.](https://ocularnebula.newgrounds.com/)

<details>

<summary> Ocular Rotation System - Comments </summary>

The general rationale behind most kicks is, "these first kicks feel most natural, any additional kicks after that serve flexibility, cool tricks and skill ceiling". However, more concrete heuristics can be stated:
- A general trend with kicks is to *prevent needless upwarping* of pieces. This simply means we first prefer rotations into a position *further down* before trying further up (foregoing nonsensical upward kicks in the first place).
- New positions must look 'visually close' to the original position. One thing observed in this system is that there are no disjoint rotation states, i.e. the old and new position always overlap in at least one tile. This heuristic also influenced the **L**/**J** rotations, always incorporating all rotations where *two* of the new pieces overlap with the old.
- The **I**, **S** and **Z** pieces are more rotationally symmetric than **T**, **L** and **J**, yielding the same visual shape whether rotated left or right.
  However, they do not have a natural 'center' in a way that would allow them to be rotated precisely in this sense, forcing us to choose their new position to be more left or right. We use this to our advantage and allow the player to have direct control over this by inputting one of the two rotation directions. This may arguably aid both directional intuition as well as allow for better finesse. It should not hurt rotational intuition ("piece stays in place if rotated 360°") as a player in a sense would never need to rotate such symmetrical pieces (especially mid-air) more than once anyway. :P


*\*Notation*: `nTlr 0-3` describes kick positions `0` to `3` when rotating a `n`orth-facing `T`-piece to the `l`eft _or_ `r`ight.

![Ocular Rotation System Heatmap](Gallery/ocular-rotation-system_16px.png)

- **O**-piece.
  - As the most symmetrical piece, having no kicks would be most natural, but also make it the only piece where rotation is 'useless'. Adding limited kicks however already turns out to be very useful to the player:
    - `nOl 0`: Simple sideways O-'roll', providing a second way to move O.
    - `nOl 1`: Allows for O-spins.
    - `nOl 2`: Also allows for O-spins and - more likely - 'rolling' the O out of a shallow well, possibly useful under high gravity.
    - `nOl 3`: While invisible on the chart, this technically allows the O to *always* successfully change its orientation upon rotation.
- **I**-piece.
  - As the longest piece, it has the most kicks to accommodate its visual range. What's special is that in its horizontal position it may rotate left or right into *any* reachable space around itself.
    - `nIl 0-7`: Rotating in it from a horizontal to a vertical position will very strongly prefer placing it in the desired direction of rotation first.
    - `nIl 8-9`: Fallback rotation states.
    - `eIl 5-6`: Allows tucking the I down flat.
    - Non-existent higher-positioned `eIl`: Intentionally left away to prevent upwarping.
- **S**/**Z**-piece.
  - The rotations for these pieces also allow for directional finesse.
    - `nSZlr`: Kept simple and symmetrical.
    - `eSr 2`|`eZl 2`: Useful and commonly applicable S/Z spin.
    - `eSl 2`|`eZr 2`: Tucking in an S/Z in this position might be handy.
- **T**-piece.
  - This piece has a relatively natural center of rotation, making for a good first rotation reposition.
    - `neswTlr 0`: 'Center' rotation.
    - `nTlr 4`: A T-'turn'.
    - `eTl 4`|`wTr 4`: A T-'insert', allowing it to warp down.
    - `sTlr 4`: A T-'turn'.
    - `sTlr 5`: This kick enables T-spin triples.
    - `wTl 3-4`|`eTr 3-4`: Two T-'turns'.
- **L**/**J**-piece.
  - Surprisingly, the most common center of rotation for L/J does not lead to a first rotation reposition where an overlap of two tiles with the original position is observed.
    - `neswLJlr 0`: 'Center' rotation.
    - `nLl 4`|`nJr 4`, `eLl 5`|`wJr 5`, `sLl 2`|`sJr 2`, `wLl 1`|`wJr 1`: Additional 'wall'-kicks that come after trying the 'center' rotation.
    - `nLr 4`|`nJl 4`, `eLr 5`|`wJl 5`, `sLr 4`|`sJl 4`, `wLr 2`|`eJl 2`: Somewhat weird kicks that are mostly included due to context and symmetry in rotation.
    - `wLl 6-7`|`eJr 6-7`: Two ways to specially tuck an L/J down.
    - `sLr 6`|`sJl 6`: Allows rotation into upright state even when resting on other blocks in a well.

</details>


## Piece Locking

The mechanics of locking down a piece on the grid can be more complicated than it might sound at first glance.

Good criteria for a locking system I can think of would be:

1. Keep players from stalling / force players to make a choice eventually.
2. Give players enough flexibility to manipulate the piece even if it's on the ground.
3. Force players to *react/input faster* on higher levels, as speed is supposed to increase.
4. Implement all these limitations as naturally/simply as possible.

So I started looking and deciding which locking system to implement;

<details>

<summary> Creating a Locking System. </summary>

*Classic lock down* is simple, but if one decreases the lock timer at higher levels (3.) then it might become exceedingly difficult for players to actually have enough time to do adjustments (2.).

<details>

<summary> Classic Lock Down </summary>

> - If the piece touches a surface
>   - start a lock timer of 500ms (\**var with lvl*).
>   - record the lowest y coordinate the piece has reached.
> - If the lock timer runs out, lock the piece immediately as soon as it touches the next surface.
> - If the piece falls below the previously lowest recorded y coordinate, reset lock timer.

</details>

*Infinite lock down* essentially mitigates the flexibility issue by saying, *"if the player manipulated his piece, give him some more time"*. 
It's very simple, but the fact that it lets players stall forever (1.) is less nice.

<details>

<summary> Infinite Lock Down </summary>

> - If the piece touches a surface
>   - start a lock timer of 500ms (\**var with lvl*).
> - If the lock timer runs out, lock the piece immediately as soon as it touches the next surface.
> - If the piece moves/rotates (change in position), reset lock timer ('move reset').

</details>

The standard recommended by the guideline is therefore *extended placement lock down*.

<details>

<summary> Extended Placement Lock Down </summary>

> - If the piece touches a surface
>   - start a lock timer of 500ms (\**var with lvl*).
>   - start counting the number of moves/rotates the player makes.
>   - record the lowest y coordinate the piece has reached.
> - If the piece moves/rotates (change in position), reset lock timer ('move reset').
> - If the number of moves reaches 15, do not reset the lock timer anymore.
> - If the lock timer runs out, lock the piece immediately as soon as it touches the next surface.
> - If the piece falls below the previously lowest recorded y coordinate, reset counted number of moves.

*(\*This probably misses a few edge cases, but you get the gist.)*

</details>

Yeah.

It's pretty flexible (2.) yet forces a decision (1.), but the 'count to 15 moves' part of this lock down seems somewhat arbitrary (4.)
<sup>*(\*Also note that after the 15 moves run out one can still manipulate the piece till lock down.)*</sup>

> **Idea.**
> 
> What if we limit the *total amount of time a piece may touch a surface* (1.) instead of number of moves/rotates (4.), though but at higher levels the piece *attempts* to lock down faster (3.), re-attempting later upon move/rotate;
> This still allows for plenty <sup>*\*technically arbitrarily many*</sup> piece manipulations (2.) while still fulfilling the other points :D

<details>

<summary> 'Timer' Extended Placement Lock Down </summary>

*Let 'ground time' denote the amount of time a piece touches a surface*

> - If the piece touches a surface
>   - start a lock timer of 500ms (\**var with lvl*).
>   - start measuring the ground time.
>   - record the lowest y coordinate the piece has reached.
> - If the piece moves/rotates (change in position), reset lock timer ('move reset').
> - If the lock timer runs out *or* the ground time reaches 2.25s, lock the piece immediately as soon as it touches the next surface.
> - If the piece falls below the previously lowest recorded y coordinate, reset the ground time.

Nice.

</details>

Although now it *may potentially* be abused by players which keep pieces in the air, only to occasionally touch down and reset the lock timer while hardly adding any ground time (note that this problem vanishes at 20G).

A small patch for this is to check the last time the piece touched the ground, and if that was, say, less than 2×(drop delay) ago, then act as if the piece had been touching ground all along. This way the piece is guaranteed to be counted as "continuously on ground" even with fast upward kicks of height ≤ 2.

</details>

In the end, a timer-based extended placement lockdown (+ ground continuity fix) is what I used.
Although there might be a nicer system somehow..


## Scoring

The exact scoring formula is given as follows:
<details>

<summary>Scoring Formula</summary>

```haskell
score_bonus = 10
            * (lines + combo - 1) ^ 2
            * maximum [1, backToBack]
            * (if spin then 4 else 1)
            * (if perfect then 100 else 1)
  where lines = "number of lines cleared simultaneously"
        spin = "piece could not move up when locking occurred"
        perfect = "board is empty after line clear"
        combo = "number of consecutive played pieces where line clear occurred"
        backToBack = "number of consecutive line clears where spin, perfect or quadruple line clear occurred"
```

</details>


<details>

<summary>Table of Example Bonuses</summary>

*A table of some example bonuses:*
| Score bonus | Action |
| -: | :- |
| +10 | Single |
| +40 | Double |
| +90 | Triple |
| +160 | Quadruple |
| +40 | ?-Spin Single |
| +160 | ?-Spin Double |
| +360 | ?-Spin Triple |
| +40 | Single (2.combo) |
| +90 | Single (3.combo) |
| +160 | Single (4.combo) |
| +90 | Double (2.combo) |
| +160 | Double (3.combo) |
| +250 | Double (4.combo) |
| +160 | Triple (2.combo) |
| +250 | Triple (3.combo) |
| +360 | Triple (4.combo) |
| +320 | Quadruple (2.B2B) |
| +480 | Quadruple (3.B2B) |
| +640 | Quadruple (4.B2B) |
| +1'000 | Perfect Single |
| +16'000 | Perfect L-Spin Double |

</details>

Coming up with a *good* [scoring system](https://tetris.wiki/Scoring#Recent_guideline_compatible_games) is easier with practical experience and playtesters.

I did actually try to come up with a new, simple, good formula, but it's tough to judge how much to reward the player for any given action *(how many points should a 'perfect clear' receive? - I've never achieved a single perfect clear in my life!)*.
The one I came up with, put mildly, *probably sucks*.

But I still allowed myself to experiment, because I really liked the idea of [rewarding all spins](https://harddrop.com/wiki/List_of_twists) (and don't understand modern Tetris' obession with T-spins when S-, Z-, L- and J-spins are also so satisfying).


## Controls

A search for the 'best' / 'most ergonomic' game keybinds was [inconclusive](https://youtube.com/watch?v=6YhkkyXydNI&t=809).
In a sample of a few dozen opinions from reddit posts there was about a 50/50 split on

| move | rotate |
| - | - |
| `a` `d` | `←` `→` |
| `←` `→` | `z` `x` |

Frequent advice I saw, "Choose what feels best for you", which sounds about right.
*(\*though some mentioned one should **not** hammer ` spacebar ` for hard drops, the only button the Guideline suggests for this action.)*


## Menu Navigation

Modeling how a TUI should handle menus and move between them was unclear initially.
Luckily, I was able to look at how [Noita](https://noitagame.com/)'s menus are connected and saw that it was quite structured:
The menus form a graph (with menus as nodes and valid transitions as directed edges), with only some menus ('pop-ups') that allow backtracking to a previous menu.

<details>

<summary>Tetrs Terminal Menu Graph (via Graphviz)</summary>

![tetrs menu graph](Gallery/tetrs_menu-graph.png)

</details>


## Miscellaneous Author Notes

In the two very intense weeks of developing this project I've had my first proper learning experiences with programming a larger Rust project, an interactive game (in the console no less), and the intricacies of modern tetrs mechanics themselves.

Gamedev-wise I can mention learning about the [modern](https://gafferongames.com/post/fix_your_timestep/) [game](http://gameprogrammingpatterns.com/game-loop.html) [loop](https://dewitters.com/dewitters-gameloop/);
Finding the proper abstraction for `Game::update` (allow arbitrary-time user input, make updates decoupled from framerate) was still hard.

Frontend-wise I may have used [Ratatui](https://crates.io/crates/ratatui/), but decided to just do some basic menus myself using the trusty [Crossterm](https://crates.io/crates/crossterm) for cross-platform terminal manipulation.
Next time I'd use a TUI crate so as to sleep more peacefully at night not having to think about the ~horrible ad-hoc code I wrote for the interface~

On the Rust side of things I learned about;
- Some [coding](https://docs.kernel.org/rust/coding-guidelines.html) [style](https://doc.rust-lang.org/nightly/style-guide/) [guidelines](https://github.com/rust-lang/rust-analyzer/blob/master/docs/dev/style.md#getters--setters) & `cargo fmt` (~`#[rustfmt::skip]`~),
- "[How to order Rust code](https://deterministic.space/how-to-order-rust-code.html)",
- introduction to [writing](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html) [documentation](https://rust-lang.github.io/api-guidelines/documentation.html) (and the fact they can [contain tested examples](https://blog.guillaume-gomez.fr/articles/2020-03-12+Guide+on+how+to+write+documentation+for+a+Rust+crate#Hiding-lines)) & `cargo doc`,
- the [`std` traits](https://rust-lang.github.io/api-guidelines/interoperability.html),
- using [serde](https://serde.rs/derive.html) a little for a hacky way to [save some structured data locally](https://stackoverflow.com/questions/62771576/how-do-i-save-structured-data-to-file),
- [conditionally derive](https://stackoverflow.com/questions/42046327/conditionally-derive-based-on-feature-flag) feature flags & `cargo check --features serde`,
- [conditionally compile](https://doc.rust-lang.org/reference/conditional-compilation.html),
- basic [file system](https://doc.rust-lang.org/std/fs/index.html) shenanigans,
- [clap](https://docs.rs/clap/latest/clap/) to parse simple command line arguments & `cargo run -- --fps=60`,
- [formatting](https://docs.rs/chrono/latest/chrono/struct.DateTime.html#method.format) the time with [chrono](https://rust-lang-nursery.github.io/rust-cookbook/datetime/parse.html#display-formatted-date-and-time) my favourite way,
- the `format!` macro (which I discovered is the analogue to Python's f-strings my beloved),
- [`debug_struct`](https://doc.rust-lang.org/std/fmt/struct.Formatter.html#method.debug_struct) proved quite helpful to ensure `Debug` for all structs,
- some [annoyances](https://sw.kovidgoyal.net/kitty/keyboard-protocol/#progressive-enhancement) with terminal emulators, including how slow they are ~on Windows~,
- the handy drop-in [`BufWriter`](https://doc.rust-lang.org/std/io/struct.BufWriter.html) wrapper to diminish flickering,
- settings a custom [panic hook](https://doc.rust-lang.org/std/panic/fn.set_hook.html) (since TUI shenanigans mess with error output),
- more practice with Rust's [module system](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html),
- multithreading with [`std::sync::mpsc`](https://doc.rust-lang.org/std/sync/mpsc/)
- [cargo workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to fully separate frontend and backend,
- [cargo git dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories) so other people *could* reuse the backend,
- learning about [cross-compilation](https://blog.logrocket.com/guide-cross-compilation-rust/#how-rust-represents-platforms) for releases,
- and as last honourable mention: Looking for a good input reading crate for *ages*, failing to get [device_query](https://crates.io/crates/device_query) to work, and settling on trusty Crossterm, which did its job perfectly and I couldn't be happier, considering how not-made-for-games consoles are.

All in all, Rust, known for its safety and performance - while still having high-level constructs like abstract datatypes - proved to be an excellent choice for this project.

Also, I'd like to appreciate how nice the name *tetrs* fits for a Rust game that does not infringe on copyright <sup>~~though there are, like, a quadrillion other [`tetrs`](https://github.com/search?q=%22tetrs%22&type=repositories)'s on GitHub, ooof~~</sup>.

- For the menu navigation graph I used [graphviz](http://magjac.com/graphviz-visual-editor/).
- For the terminal GIF recordings I used [asciinema](https://asciinema.org/) + [agg](https://github.com/asciinema/agg):
  ```bash
  agg --font-family="DejaVu Sans Mono" --line-height=1.17 --renderer=resvg --font-size=20, --fps-cap=30 --last-frame-duration=0  my_rec.cast my_rec.gif
  ```


*„Piecement Places!“* - [CTWC 2016](https://www.youtube.com/watch?v=RlnlDKznIaw&t=121).


<div  align="center">
  
`██ ▄▄▄▄ ▄█▀ ▀█▄ ▄█▄ ▄▄█ █▄▄`

</div>
