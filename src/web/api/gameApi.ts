// API client for word game backend

const API_BASE_URL = 'http://localhost:3001/api';

export interface ApiGame {
  id: string;
  date: string;
  board: ApiBoard;
  threshold_score: number;
}

export interface ApiBoard {
  tiles: ApiTile[][];
}

export interface ApiTile {
  letter: string;
  points: number;
  is_wildcard: boolean;
  row: number;
  col: number;
}

export interface ApiAnswer {
  word: string;
  score: number;
  path: ApiPosition[];
  wildcard_constraints: Record<string, string>;
}

export interface ApiPosition {
  row: number;
  col: number;
}

export interface ValidateRequest {
  word: string;
  previous_answers: ApiAnswer[];
}

export interface ValidateResponse {
  is_valid: boolean;
  score: number;
  path: ApiPosition[];
  wildcard_constraints: Record<string, string>;
  error_message: string;
}

export interface SubmitRequest {
  user_id?: string;
  answers: ApiAnswer[];
}

export interface SubmitResponse {
  user_id: string;
  total_score: number;
  stats: ApiGameStats;
}

export interface ApiGameStats {
  total_players: number;
  user_rank: number;
  percentile: number;
  average_score: number;
  highest_score: number;
}

class GameApi {
  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`;
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  async getDailyGame(date?: string): Promise<ApiGame> {
    const endpoint = date ? `/daily-game/${date}` : '/daily-game';
    return this.request<ApiGame>(endpoint);
  }

  async validateAnswer(request: ValidateRequest): Promise<ValidateResponse> {
    return this.request<ValidateResponse>('/validate', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async submitAnswers(request: SubmitRequest): Promise<SubmitResponse> {
    return this.request<SubmitResponse>('/submit', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async createUser(): Promise<{ user_id: string; cookie_token: string }> {
    return this.request<{ user_id: string; cookie_token: string }>('/user', {
      method: 'POST',
    });
  }
}

export const gameApi = new GameApi();

// Utility functions to convert between API types and frontend types
export function convertApiTileToTile(apiTile: ApiTile): import('../utils/scoring').Tile {
  return {
    letter: apiTile.letter,
    points: apiTile.points,
    isWildcard: apiTile.is_wildcard,
    row: apiTile.row,
    col: apiTile.col,
  };
}

export function convertApiBoardToBoard(apiBoard: ApiBoard): import('../utils/scoring').Tile[][] {
  return apiBoard.tiles.map(row => 
    row.map(tile => convertApiTileToTile(tile))
  );
}

export function convertApiPositionToPosition(apiPos: ApiPosition): import('../utils/scoring').Position {
  return {
    row: apiPos.row,
    col: apiPos.col,
  };
}