# 8-Puzzle Solver

Yeay, another 8-puzzle solver! _Who needs this, though?_ Well, this is just my attempt to solve 8-puzzle using A\* algorithm. Previously, I did not know that A\* algorithm can be used for this, but I was recommended a few videos in YouTube about using A\* search to solve 8-puzzle. I thought "Hmm, that's interesting." And so this is my attempt.

## Prerequisites

- [Pixi by prefix.dev](https://pixi.prefix.dev).
    - Rust is included inside. No need to instal it separately.

## Install Dependencies

1) Install Pixi's dependencies: `pixi install`. This will install Cargo and Rust language.
2) Install Cargo's dependencies: `pixi run build`.

## Run Program

`pixi run start`

## Screenshots

![Initial state of the board.](./screenshots/01_initial_state.png)
![The board when all squares all filled.](./screenshots/02_filled.png)
![Step 1 of the solution, if the solution is found.](./screenshots/03_solution_step_1.png)
![Step 2 of the solution, if the solution is found.](./screenshots/04_solution_step_2.png)
![Last step of the solution, if the solution is found.](./screenshots/05_solution_last_step.png)
![If the process takes a bit long, then the loading page will be shown.](./screenshots/06_loading.png)
![The displayed page if the solution is not found.](./screenshots/07_solution_not_found.png)
