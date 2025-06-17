export interface Position {
  row: number;
  col: number;
}

export interface Tile {
  letter: string;
  points: number;
  isWildcard: boolean;
  row: number;
  col: number;
}