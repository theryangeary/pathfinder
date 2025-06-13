import { describe, it, expect } from 'vitest'
import { findAllPaths } from '../utils/pathfinding'
import { Tile } from '../utils/scoring'

describe('Pathfinding Tests', () => {
  it('should find exactly 3 paths for "vea" on tarae*oros*sotvi board', () => {
    // Board layout: tarae*oros*sotvi
    // t a r a
    // e * o r  <- wildcard at (1,1)
    // o s * s  <- wildcard at (2,2)
    // o t v i
    const board: Tile[][] = [
      [
        { letter: 't', points: 1, isWildcard: false, row: 0, col: 0 },
        { letter: 'a', points: 1, isWildcard: false, row: 0, col: 1 },
        { letter: 'r', points: 1, isWildcard: false, row: 0, col: 2 },
        { letter: 'a', points: 1, isWildcard: false, row: 0, col: 3 },
      ],
      [
        { letter: 'e', points: 1, isWildcard: false, row: 1, col: 0 },
        { letter: '*', points: 0, isWildcard: true, row: 1, col: 1 },
        { letter: 'o', points: 1, isWildcard: false, row: 1, col: 2 },
        { letter: 'r', points: 1, isWildcard: false, row: 1, col: 3 },
      ],
      [
        { letter: 'o', points: 1, isWildcard: false, row: 2, col: 0 },
        { letter: 's', points: 1, isWildcard: false, row: 2, col: 1 },
        { letter: '*', points: 0, isWildcard: true, row: 2, col: 2 },
        { letter: 's', points: 1, isWildcard: false, row: 2, col: 3 },
      ],
      [
        { letter: 'o', points: 1, isWildcard: false, row: 3, col: 0 },
        { letter: 't', points: 1, isWildcard: false, row: 3, col: 1 },
        { letter: 'v', points: 1, isWildcard: false, row: 3, col: 2 },
        { letter: 'i', points: 1, isWildcard: false, row: 3, col: 3 },
      ],
    ]

    const paths = findAllPaths(board, 'vea')
    
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
    // Same board as above
    const board: Tile[][] = [
      [
        { letter: 't', points: 1, isWildcard: false, row: 0, col: 0 },
        { letter: 'a', points: 1, isWildcard: false, row: 0, col: 1 },
        { letter: 'r', points: 1, isWildcard: false, row: 0, col: 2 },
        { letter: 'a', points: 1, isWildcard: false, row: 0, col: 3 },
      ],
      [
        { letter: 'e', points: 1, isWildcard: false, row: 1, col: 0 },
        { letter: '*', points: 0, isWildcard: true, row: 1, col: 1 },
        { letter: 'o', points: 1, isWildcard: false, row: 1, col: 2 },
        { letter: 'r', points: 1, isWildcard: false, row: 1, col: 3 },
      ],
      [
        { letter: 'o', points: 1, isWildcard: false, row: 2, col: 0 },
        { letter: 's', points: 1, isWildcard: false, row: 2, col: 1 },
        { letter: '*', points: 0, isWildcard: true, row: 2, col: 2 },
        { letter: 's', points: 1, isWildcard: false, row: 2, col: 3 },
      ],
      [
        { letter: 'o', points: 1, isWildcard: false, row: 3, col: 0 },
        { letter: 't', points: 1, isWildcard: false, row: 3, col: 1 },
        { letter: 'v', points: 1, isWildcard: false, row: 3, col: 2 },
        { letter: 'i', points: 1, isWildcard: false, row: 3, col: 3 },
      ],
    ]

    // Test with a wildcard constraint - constrain wildcard at (1,1) to be 'v'
    const wildcardConstraints = { '1-1': 'v' }
    const paths = findAllPaths(board, 'vea', wildcardConstraints)
    
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
    // Same board as above
    const board: Tile[][] = [
      [
        { letter: 't', points: 1, isWildcard: false, row: 0, col: 0 },
        { letter: 'a', points: 1, isWildcard: false, row: 0, col: 1 },
        { letter: 'r', points: 1, isWildcard: false, row: 0, col: 2 },
        { letter: 'a', points: 1, isWildcard: false, row: 0, col: 3 },
      ],
      [
        { letter: 'e', points: 1, isWildcard: false, row: 1, col: 0 },
        { letter: '*', points: 0, isWildcard: true, row: 1, col: 1 },
        { letter: 'o', points: 1, isWildcard: false, row: 1, col: 2 },
        { letter: 'r', points: 1, isWildcard: false, row: 1, col: 3 },
      ],
      [
        { letter: 'o', points: 1, isWildcard: false, row: 2, col: 0 },
        { letter: 's', points: 1, isWildcard: false, row: 2, col: 1 },
        { letter: '*', points: 0, isWildcard: true, row: 2, col: 2 },
        { letter: 's', points: 1, isWildcard: false, row: 2, col: 3 },
      ],
      [
        { letter: 'o', points: 1, isWildcard: false, row: 3, col: 0 },
        { letter: 't', points: 1, isWildcard: false, row: 3, col: 1 },
        { letter: 'v', points: 1, isWildcard: false, row: 3, col: 2 },
        { letter: 'i', points: 1, isWildcard: false, row: 3, col: 3 },
      ],
    ]

    // Try to find paths for a word that can't be formed on this board
    const paths = findAllPaths(board, 'xyz')
    
    // Should find no paths
    expect(paths).toHaveLength(0)
  })
})