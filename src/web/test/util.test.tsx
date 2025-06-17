import { describe, it, expect } from 'vitest'
import { Tile } from '../utils/models'

export function testBoard(letters: string): Tile[][] {
  return [
      [
        { letter: letters[0], points: 1, isWildcard: false, row: 0, col: 0 },
        { letter: letters[1], points: 1, isWildcard: false, row: 0, col: 1 },
        { letter: letters[2], points: 1, isWildcard: false, row: 0, col: 2 },
        { letter: letters[3], points: 1, isWildcard: false, row: 0, col: 3 },
      ],
      [
        { letter: letters[4], points: 1, isWildcard: false, row: 1, col: 0 },
        { letter: letters[5], points: 0, isWildcard: true, row: 1, col: 1 },
        { letter: letters[6], points: 1, isWildcard: false, row: 1, col: 2 },
        { letter: letters[7], points: 1, isWildcard: false, row: 1, col: 3 },
      ],
      [
        { letter: letters[8], points: 1, isWildcard: false, row: 2, col: 0 },
        { letter: letters[9], points: 1, isWildcard: false, row: 2, col: 1 },
        { letter: letters[10], points: 0, isWildcard: true, row: 2, col: 2 },
        { letter: letters[11], points: 1, isWildcard: false, row: 2, col: 3 },
      ],
      [
        { letter: letters[12], points: 1, isWildcard: false, row: 3, col: 0 },
        { letter: letters[13], points: 1, isWildcard: false, row: 3, col: 1 },
        { letter: letters[14], points: 1, isWildcard: false, row: 3, col: 2 },
        { letter: letters[15], points: 1, isWildcard: false, row: 3, col: 3 },
      ],
    ]
}

describe('Test Utils', () => {
  it('should create a test board with correct structure', () => {
    const board = testBoard('abcdefghijklmnop')
    expect(board).toHaveLength(4)
    expect(board[0]).toHaveLength(4)
    expect(board[0][0].letter).toBe('a')
    expect(board[3][3].letter).toBe('p')
  })
})
