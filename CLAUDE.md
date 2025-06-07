# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a web-based daily word puzzle game that blends Boggle and Scrabble mechanics. Players find words on a 4x4 grid by connecting adjacent tiles, with letter point values based on frequency and wildcard tiles that can represent any letter.

## Game Architecture

### Core Components Needed
- **Board Generation**: 4x4 grid with letter tiles and wildcard placement logic
- **Pathfinding System**: Validates word paths through adjacent/diagonal connections
- **Scoring Engine**: Calculates points using letter frequency-based values
- **Wildcard Constraint System**: Manages wildcard tile letter assignments across multiple answers
- **Word Validation**: Checks against word list and path validity
- **UI State Management**: Handles progressive answer unlocking and visual feedback

### Key Game Rules
- 5 answers required, unlocked sequentially
- Wildcard tiles: 2 non-adjacent interior tiles with 0 points
- Path duplicate resolution: prefer no wildcards → fewer wildcards → fewer diagonal moves → later diagonal moves
- Visual feedback: red X (invalid), green checkmark (valid), path highlighting, score display

### Wildcard Constraint Minimization
The game applies constraint minimization principles to preserve maximum flexibility for future answers:

1. **Path Highlighting Rules**:
   - If ANY paths use 0 wildcards, highlight ALL valid paths (including wildcard paths)
   - If NO paths use 0 wildcards, apply selective highlighting based on necessity

2. **Wildcard Necessity Rules**:
   - Rule 2a: Don't use wildcards if non-wildcard alternatives exist for the same letter
   - Rule 2b: Consider both wildcards if both are accessible and can reach the next letter

3. **Constraint Preference**: Choose paths that require the fewest wildcard constraints, leaving maximum flexibility for future word inputs

### Wildcard Notation System
Wildcard tiles display different notations based on current constraints:

- **Single letter** (e.g., "E"): Wildcard must be that specific letter
- **Letter / \*** (e.g., "S / \*"): Wildcard could be that letter, but the other wildcard remains free for other uses
- **Letter1 / Letter2** (e.g., "E / L"): Wildcard must be one of two letters, with paired constraints between wildcards

This notation system communicates constraint relationships clearly while preserving flexibility.

### Letter Point Values
Points calculated as: `floor(log2(freq_of_e / freq_of_letter)) + 1`
Wildcard tiles always worth 0 points.

## Project Structure

```
src/
├── web/                   # Frontend (React/TypeScript)
│   ├── components/        # React components (Board, Tile, AnswerSection, etc.)
│   ├── api/              # Backend API client (gameApi.ts)
│   ├── hooks/            # React hooks (useUser.ts)
│   ├── utils/            # Game logic utilities (pathfinding, scoring, board generation)
│   └── data/             # Static data files (wordList.ts)
└── api/                  # Backend (Rust)
    ├── src/              # Rust source code
    │   ├── db/           # Database layer (models, repository)
    │   ├── game/         # Game logic (board, scoring, pathfinding)
    │   └── http_api.rs   # HTTP API endpoints
    ├── proto/            # gRPC protocol definitions
    ├── migrations/       # Database migrations
    └── wordlist          # Word validation data
```

## Development Status

The project is fully implemented with:
- ✅ Complete React frontend with TypeScript
- ✅ Rust backend with HTTP API and gRPC services  
- ✅ SQLite database with automated migrations
- ✅ End-to-end integration working
- ✅ User session management
- ✅ Daily puzzle generation with quality checking

## Development Servers

### Frontend Development
```bash
npm run dev
```
Starts Vite development server with hot reload at `http://localhost:5173`

### Backend Development  
```bash
cd src/api
cargo run
```
Starts backend servers:
- HTTP API: `http://localhost:3001`
- gRPC: `http://localhost:50051`

### Integration Testing
```bash
./test_integration.sh
```

## Claude Code Instructions

- Never run a webserver, I will run a webserver in another window. Just let me know what to run, and in the meantime you can run any commands other than the webserver to verify the code changes.
- Frontend source is in `src/web/`, backend source is in `src/api/`
- Use the HTTP API client in `src/web/api/gameApi.ts` for frontend-backend communication