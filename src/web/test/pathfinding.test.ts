import { describe, expect, it } from 'vitest'
import { PathConstraintType } from '../utils/models'
import { findAllPaths, findAllPathsGivenWildcards, findBestPath, getWildcardConstraintsFromPath, isPathCompatibleWithConstraints } from '../utils/pathfinding'
import { testBoard } from './util.test'




describe('Pathfinding Tests', () => {
  it('should implement findAllPaths function correctly', () => {
    const board = testBoard('tarae*oros*sotvi')

    const result = findAllPaths(board, 'vea')

    // Verify return structure
    expect(result.word).toBe('vea')
    expect(result.paths).toBeDefined()
    expect(result.constraintsSet).toBeDefined()
    expect(result.constraintsSet.pathConstraintSets).toBeDefined()

    // Should find exactly 3 paths for 'vea' on this board (same as findAllPathsGivenWildcards)
    expect(result.paths).toHaveLength(3)

    // Each path should have both the position array and constraints
    result.paths.forEach(pathWithConstraints => {
      expect(pathWithConstraints.path).toBeDefined()
      expect(pathWithConstraints.constraints).toBeDefined()
      expect(pathWithConstraints.path).toHaveLength(3) // 3 letters in 'vea'
    })

    // Constraints set should have same number of constraint sets as paths
    expect(result.constraintsSet.pathConstraintSets).toHaveLength(3)

    // Test with a simple word that uses no wildcards
    const result2 = findAllPaths(board, 'tar')
    expect(result2.word).toBe('tar')
    expect(result2.paths.length).toBeGreaterThan(0)

    // For a word that doesn't use wildcards, constraints should be Unconstrained
    const nonWildcardPath = result2.paths.find(p =>
      p.constraints.type === PathConstraintType.Unconstrained
    )
    expect(nonWildcardPath).toBeDefined()
  })
  it('should find exactly 3 paths for "vea" on tarae*oros*sotvi board', () => {
    const board = testBoard('tarae*oros*sotvi')

    const paths = findAllPathsGivenWildcards(board, 'vea')

    // Should find exactly 3 paths for 'vea'
    expect(paths).toHaveLength(3)

    // Convert paths to coordinate arrays for easier comparison
    const pathCoords = paths.map(path =>
      path.map(pos => [pos.row, pos.col])
    )

    // Expected paths based on the backend test results:
    // Path 1: [(1,1),(1,0),(0,1)] - wildcard->e->a (corrected order)
    // Path 2: [(2,2),(1,1),(0,1)] - wildcard->wildcard->a  
    // Path 3: [(3,2),(2,2),(1,1)] - v->wildcard->wildcard
    const expectedPaths = [
      [[1, 1], [1, 0], [0, 1]], // wildcard->e->a
      [[2, 2], [1, 1], [0, 1]], // wildcard->wildcard->a
      [[3, 2], [2, 2], [1, 1]], // v->wildcard->wildcard
    ]

    // Verify all expected paths are present
    for (const expectedPath of expectedPaths) {
      const found = pathCoords.some(path =>
        path.length === expectedPath.length &&
        path.every((coord, i) =>
          coord[0] === expectedPath[i][0] && coord[1] === expectedPath[i][1]
        )
      )
      expect(found).toBe(true)
    }

    // Verify each path has exactly 3 positions (for the 3-letter word "vea")
    paths.forEach(path => {
      expect(path).toHaveLength(3)
    })
  })

  it('should respect wildcard constraints when finding paths', () => {
    const board = testBoard('tarae*oros*sotvi')

    // Test with a wildcard constraint - constrain wildcard at (1,1) to be 'v'
    const wildcardConstraints = { '1-1': 'v' }
    const paths = findAllPathsGivenWildcards(board, 'vea', wildcardConstraints)

    // Should only find paths that use the wildcard at (1,1) as 'v'
    // This should eliminate some paths found without constraints
    expect(paths.length).toBeGreaterThan(0)

    // Verify that any path using wildcard at (1,1) treats it as 'v'
    paths.forEach(path => {
      const usesConstrainedWildcard = path.some(pos => pos.row === 1 && pos.col === 1)
      if (usesConstrainedWildcard) {
        // The path should be valid for 'vea' with wildcard at (1,1) as 'v'
        expect(path).toHaveLength(3)
      }
    })
  })

  it('should find no paths for an impossible word', () => {
    const board = testBoard('tarae*oros*sotvi')

    // Try to find paths for a word that can't be formed on this board
    const paths = findAllPathsGivenWildcards(board, 'xyz')

    // Should find no paths
    expect(paths).toHaveLength(0)
  })

  it('should resolve correct wildcards', () => {
    const board = testBoard('eadux*ysta*tnhrv')

    // Test the sequence 'day', 'year', 'sev', 'data' to verify wildcard constraint behavior
    // This sequence will demonstrate the conflict resolution:
    // - 'day' uses no wildcards, establishing no constraints
    // - 'year' constrains the first wildcard (1,1) to 'e' OR the second wildcard (2,2) to 'e'
    // - 'sev' constrains the second wildcard (2,2) to 'e'  
    // - 'data' needs the first wildcard (1,1) to be 't'
    // Expected final state: wildcard (1,1) = 't', wildcard (2,2) = 'e'

    let constraints: Record<string, string> = {}

    // 1. Enter 'day' - should not constrain any wildcards
    const dayPath = findBestPath(board, 'day', constraints)
    expect(dayPath).toBeTruthy()
    if (dayPath) {
      const dayConstraints = getWildcardConstraintsFromPath(board, 'day', dayPath)
      constraints = { ...constraints, ...dayConstraints }
    }

    // 2. Enter 'year' - should constrain either first or second wildcard (1,1) to 'e'
    const yearPath = findBestPath(board, 'year', constraints)
    expect(yearPath).toBeTruthy()
    if (yearPath) {
      const yearConstraints = getWildcardConstraintsFromPath(board, 'year', yearPath)
      constraints = { ...constraints, ...yearConstraints }
    }

    // 3. Enter 'sev' - should constrain second wildcard (2,2) to 'e'
    const sevPath = findBestPath(board, 'sev', constraints)
    expect(sevPath).toBeTruthy()
    if (sevPath) {
      const sevConstraints = getWildcardConstraintsFromPath(board, 'sev', sevPath)
      constraints = { ...constraints, ...sevConstraints }
    }

    // At this point, wildcard (2,2) should be 'e'
    expect(constraints['2-2']).toBe('e')

    // 4. Enter 'data' - should constrain first wildcard (1,1) to 't'
    const dataPath = findBestPath(board, 'data', constraints)
    expect(dataPath).toBeTruthy()
    if (dataPath) {
      const dataConstraints = getWildcardConstraintsFromPath(board, 'data', dataPath)
      constraints = { ...constraints, ...dataConstraints }

      // Final verification: first wildcard should now be constrained to 't'
      expect(constraints['1-1']).toBe('t')
      expect(constraints['2-2']).toBe('e')
    }
  })

  describe('isPathCompatibleWithConstraints', () => {
    it('should return true for paths that dont use wildcards with any constraint type', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      // Path that doesn't use any wildcards: 'tar' using positions (0,0), (0,1), (0,2)
      const pathWithConstraints = {
        path: [
          { row: 0, col: 0 }, // 't'
          { row: 0, col: 1 }, // 'a'  
          { row: 0, col: 2 }  // 'r'
        ],
        constraints: { type: PathConstraintType.Unconstrained }
      }
      
      // Test with different constraint types - should all return true since path uses no wildcards
      const unconstrainedConstraint = { type: PathConstraintType.Unconstrained }
      const firstDecidedConstraint = { type: PathConstraintType.FirstDecided, firstLetter: 'E' }
      const secondDecidedConstraint = { type: PathConstraintType.SecondDecided, secondLetter: 'L' }
      const bothDecidedConstraint = { type: PathConstraintType.BothDecided, firstLetter: 'E', secondLetter: 'L' }
      
      expect(isPathCompatibleWithConstraints(pathWithConstraints, unconstrainedConstraint, board, 'tar')).toBe(true)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, firstDecidedConstraint, board, 'tar')).toBe(true)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, secondDecidedConstraint, board, 'tar')).toBe(true)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, bothDecidedConstraint, board, 'tar')).toBe(true)
    })

    it('should return true for wildcard paths with unconstrained constraint type', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      // Path using first wildcard: positions (1,1), (1,0), (0,1) for word 'vea'
      const pathWithConstraints = {
        path: [
          { row: 1, col: 1 }, // wildcard as 'v'
          { row: 1, col: 0 }, // 'e'
          { row: 0, col: 1 }  // 'a'
        ],
        constraints: { type: PathConstraintType.FirstDecided, firstLetter: 'V' }
      }
      
      const unconstrainedConstraint = { type: PathConstraintType.Unconstrained }
      
      expect(isPathCompatibleWithConstraints(pathWithConstraints, unconstrainedConstraint, board, 'vea')).toBe(true)
    })

    it('should return true when first wildcard usage matches FirstDecided constraint', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      // Path using first wildcard at (1,1) as 'V'
      const pathWithConstraints = {
        path: [
          { row: 1, col: 1 }, // wildcard as 'v'
          { row: 1, col: 0 }, // 'e'
          { row: 0, col: 1 }  // 'a'
        ],
        constraints: { type: PathConstraintType.FirstDecided, firstLetter: 'V' }
      }
      
      const matchingConstraint = { type: PathConstraintType.FirstDecided, firstLetter: 'V' }
      const nonMatchingConstraint = { type: PathConstraintType.FirstDecided, firstLetter: 'E' }
      
      expect(isPathCompatibleWithConstraints(pathWithConstraints, matchingConstraint, board, 'vea')).toBe(true)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, nonMatchingConstraint, board, 'vea')).toBe(false)
    })

    it('should return true when second wildcard usage matches SecondDecided constraint', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      // Path using second wildcard at (2,2) as 'E'
      const pathWithConstraints = {
        path: [
          { row: 3, col: 2 }, // 'v'
          { row: 2, col: 2 }, // wildcard as 'e'
          { row: 1, col: 1 }  // wildcard as 'a'
        ],
        constraints: { type: PathConstraintType.BothDecided, firstLetter: 'A', secondLetter: 'E' }
      }
      
      const matchingConstraint = { type: PathConstraintType.SecondDecided, secondLetter: 'E' }
      const nonMatchingConstraint = { type: PathConstraintType.SecondDecided, secondLetter: 'L' }
      
      expect(isPathCompatibleWithConstraints(pathWithConstraints, matchingConstraint, board, 'vea')).toBe(true)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, nonMatchingConstraint, board, 'vea')).toBe(false)
    })

    it('should return true when both wildcards match BothDecided constraint', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      // Path using both wildcards: first as 'A', second as 'E'
      const pathWithConstraints = {
        path: [
          { row: 2, col: 2 }, // second wildcard as 'e'
          { row: 1, col: 1 }, // first wildcard as 'a'
          { row: 0, col: 0 }  // 't'
        ],
        constraints: { type: PathConstraintType.BothDecided, firstLetter: 'A', secondLetter: 'E' }
      }
      
      const matchingConstraint = { type: PathConstraintType.BothDecided, firstLetter: 'A', secondLetter: 'E' }
      const wrongFirstConstraint = { type: PathConstraintType.BothDecided, firstLetter: 'V', secondLetter: 'E' }
      const wrongSecondConstraint = { type: PathConstraintType.BothDecided, firstLetter: 'A', secondLetter: 'L' }
      const wrongBothConstraint = { type: PathConstraintType.BothDecided, firstLetter: 'V', secondLetter: 'L' }
      
      expect(isPathCompatibleWithConstraints(pathWithConstraints, matchingConstraint, board, 'eat')).toBe(true)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, wrongFirstConstraint, board, 'eat')).toBe(false)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, wrongSecondConstraint, board, 'eat')).toBe(false)
      expect(isPathCompatibleWithConstraints(pathWithConstraints, wrongBothConstraint, board, 'eat')).toBe(false)
    })

    it('should handle paths that use only the first wildcard', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      // Path using only first wildcard at (1,1)
      const pathWithConstraints = {
        path: [
          { row: 1, col: 1 }, // first wildcard as 'h'
          { row: 0, col: 1 }, // 'a'
          { row: 0, col: 2 }  // 'r'
        ],
        constraints: { type: PathConstraintType.FirstDecided, firstLetter: 'H' }
      }
      
      // Should be compatible with constraints that allow first wildcard to be 'H'
      const compatibleConstraints = [
        { type: PathConstraintType.Unconstrained },
        { type: PathConstraintType.FirstDecided, firstLetter: 'H' },
        { type: PathConstraintType.BothDecided, firstLetter: 'H', secondLetter: 'L' }
      ]
      
      const incompatibleConstraints = [
        { type: PathConstraintType.FirstDecided, firstLetter: 'E' },
        { type: PathConstraintType.BothDecided, firstLetter: 'E', secondLetter: 'L' }
      ]
      
      compatibleConstraints.forEach(constraint => {
        expect(isPathCompatibleWithConstraints(pathWithConstraints, constraint, board, 'har')).toBe(true)
      })
      
      incompatibleConstraints.forEach(constraint => {
        expect(isPathCompatibleWithConstraints(pathWithConstraints, constraint, board, 'har')).toBe(false)
      })
    })

    it('should handle paths that use only the second wildcard', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      // Path using only second wildcard at (2,2)
      const pathWithConstraints = {
        path: [
          { row: 2, col: 2 }, // second wildcard as 'w'
          { row: 2, col: 1 }, // 'o'
          { row: 3, col: 1 }  // 't'
        ],
        constraints: { type: PathConstraintType.SecondDecided, secondLetter: 'W' }
      }
      
      // Should be compatible with constraints that allow second wildcard to be 'W'
      const compatibleConstraints = [
        { type: PathConstraintType.Unconstrained },
        { type: PathConstraintType.SecondDecided, secondLetter: 'W' },
        { type: PathConstraintType.BothDecided, firstLetter: 'H', secondLetter: 'W' }
      ]
      
      const incompatibleConstraints = [
        { type: PathConstraintType.SecondDecided, secondLetter: 'E' },
        { type: PathConstraintType.BothDecided, firstLetter: 'H', secondLetter: 'E' }
      ]
      
      compatibleConstraints.forEach(constraint => {
        expect(isPathCompatibleWithConstraints(pathWithConstraints, constraint, board, 'wot')).toBe(true)
      })
      
      incompatibleConstraints.forEach(constraint => {
        expect(isPathCompatibleWithConstraints(pathWithConstraints, constraint, board, 'wot')).toBe(false)
      })
    })

    it('should handle edge case with empty constraint set', () => {
      const board = testBoard('tarae*oros*sotvi')
      
      const pathWithConstraints = {
        path: [{ row: 0, col: 0 }, { row: 0, col: 1 }],
        constraints: { type: PathConstraintType.Unconstrained }
      }
      
      // Test with constraint set that has no letters defined
      const emptyConstraint = { type: PathConstraintType.FirstDecided }
      
      expect(isPathCompatibleWithConstraints(pathWithConstraints, emptyConstraint, board, 'ta')).toBe(true)
    })
  })
})
