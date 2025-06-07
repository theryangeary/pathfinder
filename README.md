# Word Game - Daily Puzzle

A web-based daily word puzzle game that blends Boggle and Scrabble mechanics. Players find words on a 4x4 grid by connecting adjacent tiles, with letter point values based on frequency and wildcard tiles.

## Project Structure

```
src/
├── web/                   # Frontend (React/TypeScript)
│   ├── components/        # React components
│   ├── api/              # Backend API client
│   ├── hooks/            # React hooks
│   ├── utils/            # Game logic utilities
│   └── data/             # Static data files
└── api/                  # Backend (Rust)
    ├── src/              # Rust source code
    ├── migrations/       # Database migrations
    └── wordlist          # Word validation data
```

## Getting Started

### Prerequisites
- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- SQLite (for local database)

### Development Setup

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Start the backend API:**
   ```bash
   cd src/api
   cargo run
   ```
   This starts the HTTP API server on `http://localhost:3001`

3. **Start the frontend development server:**
   ```bash
   npm run dev
   ```
   Access the game at `http://localhost:5173`

### Running Tests

**Integration test:**
```bash
./test_integration.sh
```

**Backend unit tests:**
```bash
cd src/api
cargo test
```

## Game Features

- **Daily Puzzles**: New 4x4 word grid every day
- **Wildcard System**: Special tiles that can represent any letter
- **Progressive Unlocking**: 5 answers required, unlocked sequentially
- **Smart Pathfinding**: Optimal path selection with constraint minimization
- **Scoring System**: Letter frequency-based point values
- **Offline Support**: Fallback to local game generation

## Architecture

- **Frontend**: React with TypeScript, Vite build system
- **Backend**: Rust with Axum HTTP framework
- **Database**: SQLite with SQLx for async operations
- **API**: RESTful HTTP/JSON endpoints
- **Scheduling**: Automated daily puzzle generation

For detailed implementation status, see [BACKEND_STATUS.md](BACKEND_STATUS.md).
