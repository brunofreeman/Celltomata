# Celltomata

## Inspiration
We were both interested in broadening our horizons in the field of web development and settled on a concept that we thought was fun and doable (with plenty of room for improvements post-hackathon)!

## What it does
[Celltomata](http://celltomata.tech/) (derived from "cellular automata") is the marriage of Conway's Game of Life (or cellular automata in general) and the popular genre of ".io" games, such as [Agar.io](https://agar.io/) and [Slither.io](http://slither.io/).

Our vision of Celltomata is roughly is as follows. Players are given periods of time to fill cells of a grid in with various types of cellular automata that each behave differently. There are then periods of time during which the players' configurations of cells are left to their own devices and evolve according to their programmed behavior. The goal is to set up a colony that will grow large while also attacking and killing off the cells of other colonies.

In its current state, [Celltomata](http://celltomata.tech/) lets users submit a username and be spawned into a single shared grid with other players. The player then can spend a set amount of points on a base configuration, which is then left to evolve on its own after it is created. We see this project as a successful proof-of-concept for a game that we are very excited to one day develop to completion.

## How we built it
An was the backend lead and Bruno was the frontend lead. The project utilizes Rust, JavaScript, HTML, CSS, AWS, and NGINX.

## Challenges we ran into
We had to forgo some of our original goals (such as a hexagonal grid) due to the short timeframe of the hackathon.

## Accomplishments that we're proud of
When we first got the frontend and backend of the codebase to communicate, we were both very excited!

## What we learned
We both learned a great deal about web hosting and connecting frontends and backends through WebSockets.

## What's next for Celltomata
There's a lot of features/polish that we envisioned but couldn't feasibly implement in one weekend by two people. We are excited to continue to work on Celltomata and see it through to our complete vision, which includes:
- Flush out the basic functionality of the game (fix the blinking issue on redraw, test multiplayer capabilities, improve the current implementations of cell selecting and game flow)
- Add bots so low numbers of players can still play
- Possibly using a hex grid instead of a square grid
- Analyzing and reevaluating the rules and stats to make the game more balanced
- Have players join a lobby and start a game all at once (the grid size would scale for the number of players)
- Better aesthetics all around: smoother movement, fancier CSS, improved layout, and sprites
- Code optimizations (e.g. being smarter about requests to the backend for cell information and drawing to the canvas)