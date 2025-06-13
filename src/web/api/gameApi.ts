// API client for word game backend

const API_BASE_URL = import.meta.env.PROD 
  ? '/api'  // In production, use relative path (nginx proxy)
  : 'http://localhost:3001/api';  // In development, use direct backend URL

export interface ApiGame {
  id: string;
  date: string;
  board: ApiBoard;
  threshold_score: number;
  sequence_number: number;
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
  cookie_token?: string;
  answers: ApiAnswer[];
  game_id: string;
}

export interface UpdateProgressRequest {
  user_id?: string;
  cookie_token?: string;
  answers: ApiAnswer[];
  game_id: string;
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

export interface GameEntryResponse {
  answers: ApiAnswer[];
  completed: boolean;
  total_score: number;
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

  async getDailyGame(): Promise<ApiGame> {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, '0');
    const day = String(now.getDate()).padStart(2, '0');
    const currentDate = `${year}-${month}-${day}`;
    return this.request<ApiGame>(`/game/date/${currentDate}`);
  }

  async getGameByDate(date: string): Promise<ApiGame> {
    // Validate that the requested date is not in the future (client-side check)
    const requestedDate = new Date(date + 'T00:00:00');
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    
    if (requestedDate > today) {
      throw new Error('Cannot load puzzles from future dates');
    }
    
    return this.request<ApiGame>(`/game/date/${date}`);
  }

  async getGameBySequence(sequenceNumber: number): Promise<ApiGame> {
    // Frontend validation: prevent obviously invalid sequence numbers
    if (sequenceNumber < 1) {
      throw new Error('Sequence number must be at least 1');
    }
    
    // Note: Additional server-side validation will be added to prevent future puzzles
    return this.request<ApiGame>(`/game/sequence/${sequenceNumber}`);
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

  async updateProgress(request: UpdateProgressRequest): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>('/update-progress', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async createUser(): Promise<{ user_id: string; cookie_token: string }> {
    return this.request<{ user_id: string; cookie_token: string }>('/user', {
      method: 'POST',
    });
  }

  async getGameEntry(gameId: string, userId?: string, cookieToken?: string): Promise<GameEntryResponse | null> {
    const params = new URLSearchParams();
    if (userId) params.append('user_id', userId);
    if (cookieToken) params.append('cookie_token', cookieToken);
    
    const query = params.toString() ? `?${params.toString()}` : '';
    return this.request<GameEntryResponse | null>(`/game-entry/${gameId}${query}`);
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
