import { describe, expect, it } from 'vitest'
import {
  convertConstraintSetsToConstraints
} from '../utils/constraintResolution'
import { Tile } from '../utils/models'
import {
  findAllPaths,
  findBestPath,
  getWildcardConstraintsFromPath,
  isFirstWildcard
} from '../utils/pathfinding'
import { validateAllAnswers } from '../utils/validation'

// Test the specific wildcard bug with 'fu' on board 'igsli*tcus*feeis'
describe('fu wildcard bug investigation', () => {
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

  it('should find all possible paths for "fu"', () => {
    const answer = findAllPaths(board, 'fu')
    
    console.log('All paths found for "fu":')
    answer.paths.forEach((pathWithConstraints, i) => {
      console.log(`Path ${i + 1}:`, pathWithConstraints.path.map(pos => `(${pos.row},${pos.col})`).join(' -> '))
      console.log(`  Constraints:`, pathWithConstraints.constraints)
      console.log(`  Uses tiles:`, pathWithConstraints.path.map(pos => {
        const tile = board[pos.row][pos.col]
        return tile.isWildcard ? `*${isFirstWildcard(pos) ? '(first)' : '(second)'}` : tile.letter
      }).join(' -> '))
    })
    
    expect(answer.paths.length).toBe(4)
  })

  it('should select best path for "fu"', () => {
    const bestPath = findBestPath(board, 'fu')
    console.log('\nBest path selected:', bestPath?.map(pos => `(${pos.row},${pos.col})`).join(' -> '))
    
    expect(bestPath).toStrictEqual([{row: 2, col: 3}, {row: 2, col: 2}])
  })

  it('should validate "fu" correctly', () => {
    const validation = validateAllAnswers(board, ['fu'], true, () => true)
    
    console.log('\nValidation result:')
    console.log('Valid answers:', validation.validAnswers)
    console.log('Constraint sets:', validation.constraintSets)
    
    expect(validation.validAnswers[0]).toBe(true)
  })

  it('should convert constraints to display format correctly', () => {
    const validation = validateAllAnswers(board, ['fu'], true, () => true)
    const displayConstraints = convertConstraintSetsToConstraints(validation.constraintSets, board)
    
    console.log('\nDisplay constraints:')
    console.log(displayConstraints)
    
    // Check if the bug is present - first wildcard showing '*' but representing 'f'
    // while second wildcard gets constrained to 'u'
    expect(displayConstraints).toBeDefined()
    
    // Log detailed analysis
    console.log('\nDetailed analysis:')
    console.log('First wildcard (1,1) constraint:', displayConstraints['1-1'])
    console.log('Second wildcard (2,2) constraint:', displayConstraints['2-2'])
  })

  it('should extract constraints from path correctly', () => {
    const answer = findAllPaths(board, 'fu')
    
    answer.paths.forEach((pathWithConstraints, i) => {
      const extractedConstraints = getWildcardConstraintsFromPath(board, 'fu', pathWithConstraints.path)
      console.log(`\nPath ${i + 1} extracted constraints:`, extractedConstraints)
      console.log(`  Original path constraints:`, pathWithConstraints.constraints)
      
      expect(extractedConstraints).toEqual(pathWithConstraints.constraints)
    })
  })
})
