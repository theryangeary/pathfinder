import { describe, expect, it } from 'vitest'
import { scoreAnswerGroup } from '../utils/scoring'
import { Tile } from '../utils/models'

describe('scoring debug for fu bug', () => {
  // Create board for 'igsli*tcus*feeis'
  const board: Tile[][] = [
    [
      { letter: 'i', points: 1, isWildcard: false, row: 0, col: 0 },
      { letter: 'g', points: 2, isWildcard: false, row: 0, col: 1 },
      { letter: 's', points: 1, isWildcard: false, row: 0, col: 2 },
      { letter: 'l', points: 1, isWildcard: false, row: 0, col: 3 }
    ],
    [
      { letter: 'i', points: 1, isWildcard: false, row: 1, col: 0 },
      { letter: '*', points: 0, isWildcard: true, row: 1, col: 1 },  // First wildcard
      { letter: 't', points: 1, isWildcard: false, row: 1, col: 2 },
      { letter: 'c', points: 3, isWildcard: false, row: 1, col: 3 }
    ],
    [
      { letter: 'u', points: 1, isWildcard: false, row: 2, col: 0 },
      { letter: 's', points: 1, isWildcard: false, row: 2, col: 1 },
      { letter: '*', points: 0, isWildcard: true, row: 2, col: 2 },  // Second wildcard
      { letter: 'f', points: 4, isWildcard: false, row: 2, col: 3 }
    ],
    [
      { letter: 'e', points: 1, isWildcard: false, row: 3, col: 0 },
      { letter: 'e', points: 1, isWildcard: false, row: 3, col: 1 },
      { letter: 'i', points: 1, isWildcard: false, row: 3, col: 2 },
      { letter: 's', points: 1, isWildcard: false, row: 3, col: 3 }
    ]
  ]

  it('should score fu correctly and show why constraint mismatch occurs', () => {
    const result = scoreAnswerGroup(['fu'], board)
    
    console.log('Scoring result for "fu":')
    console.log('Scores:', result.scores)
    console.log('Optimal constraint sets:', result.optimalConstraintSets)
    
    // The score should be 4 + 1 = 5 for path f(2,3) -> u(2,0) which doesn't use wildcards
    // OR 0 + 1 = 1 for any wildcard path
    
    // But the bug is that we're getting a SecondDecided constraint set which indicates
    // that the scoring is choosing a path that uses the second wildcard as 'u',
    // while the pathfinding might be choosing a different path
    
    expect(result.scores.fu).toBeGreaterThan(0)
  })
})