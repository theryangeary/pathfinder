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

export enum PathConstraintType {
  Unconstrained = 'Unconstrained',
  FirstDecided = 'FirstDecided',
  SecondDecided = 'SecondDecided',
  BothDecided = 'BothDecided'
}

export interface PathConstraintSet {
  type: PathConstraintType;
  firstLetter?: string;
  secondLetter?: string;
}

export interface AnswerGroupConstraintSet {
  pathConstraintSets: PathConstraintSet[];
}
