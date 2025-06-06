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

### Letter Point Values
Points calculated as: `floor(log2(freq_of_e / freq_of_letter)) + 1`
Wildcard tiles always worth 0 points.

## Development Notes

The SPEC.md contains detailed game rules and UI requirements. The project currently has no implementation - tech stack is open but vanilla JavaScript components are preferred unless frameworks provide meaningful simplification.

Reference implementations in `../wordgame` (Python CLI) and `../word-game` (partial Rust web) exist but have limitations, particularly around wildcard handling.

## Development Server

To run the development server for testing changes:

```bash
npm run dev
```

This starts the Vite development server with hot reload enabled. The game will be available at `http://localhost:5173` by default.

## Claude Code Instructions

- Never run a webserver, I will run a webserver in another window. Just let me know what to run, and in the meantime you can run any commands other than the webserver to verify the code changes.