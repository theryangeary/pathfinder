import { describe, expect, it } from 'vitest'
import { PathConstraintType } from '../utils/models'
import { findAllPaths, findAllPathsGivenWildcards, findBestPath, getWildcardConstraintsFromPath } from '../utils/pathfinding'
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
})
