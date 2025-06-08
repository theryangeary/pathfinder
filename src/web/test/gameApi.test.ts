import { describe, it, expect, vi, beforeEach, Mock } from 'vitest'
import { gameApi } from '../api/gameApi'

const mockFetch = global.fetch as Mock

describe('GameApi', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('getGameBySequence', () => {
    it('should make correct API call for sequence number', async () => {
      const mockGameData = {
        id: 'test-game-id',
        date: '2025-06-08',
        sequence_number: 5,
        threshold_score: 40,
        board: {
          tiles: [[
            { letter: 't', points: 1, is_wildcard: false, row: 0, col: 0 },
            { letter: 'e', points: 1, is_wildcard: false, row: 0, col: 1 },
            { letter: 's', points: 1, is_wildcard: false, row: 0, col: 2 },
            { letter: 't', points: 1, is_wildcard: false, row: 0, col: 3 },
          ]],
        },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockGameData,
      })

      const result = await gameApi.getGameBySequence(5)

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/game/sequence/5',
        {
          headers: {
            'Content-Type': 'application/json',
          },
        }
      )
      expect(result).toEqual(mockGameData)
    })

    it('should handle different sequence numbers', async () => {
      const testCases = [1, 10, 42, 999]

      for (const sequenceNumber of testCases) {
        const mockGameData = {
          id: `test-game-${sequenceNumber}`,
          date: '2025-06-08',
          sequence_number: sequenceNumber,
          threshold_score: 40,
          board: { tiles: [] },
        }

        mockFetch.mockResolvedValueOnce({
          ok: true,
          json: async () => mockGameData,
        })

        const result = await gameApi.getGameBySequence(sequenceNumber)

        expect(mockFetch).toHaveBeenCalledWith(
          `http://localhost:3001/api/game/sequence/${sequenceNumber}`,
          {
            headers: {
              'Content-Type': 'application/json',
            },
          }
        )
        expect(result.sequence_number).toBe(sequenceNumber)
        expect(result.id).toBe(`test-game-${sequenceNumber}`)
      }
    })

    it('should throw error when API returns 404', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found',
      })

      await expect(gameApi.getGameBySequence(999)).rejects.toThrow(
        'API request failed: 404 Not Found'
      )

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/game/sequence/999',
        {
          headers: {
            'Content-Type': 'application/json',
          },
        }
      )
    })

    it('should throw error when API returns 500', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
      })

      await expect(gameApi.getGameBySequence(1)).rejects.toThrow(
        'API request failed: 500 Internal Server Error'
      )
    })

    it('should handle network errors', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      await expect(gameApi.getGameBySequence(1)).rejects.toThrow('Network error')
    })
  })

  describe('getDailyGame', () => {
    it('should call correct endpoint without date parameter', async () => {
      const mockGameData = {
        id: 'daily-game-id',
        date: '2025-06-08',
        sequence_number: 1,
        threshold_score: 40,
        board: { tiles: [] },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockGameData,
      })

      const result = await gameApi.getDailyGame()

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/game',
        {
          headers: {
            'Content-Type': 'application/json',
          },
        }
      )
      expect(result).toEqual(mockGameData)
    })

    it('should call correct endpoint with date parameter', async () => {
      const mockGameData = {
        id: 'date-game-id',
        date: '2025-06-07',
        sequence_number: 2,
        threshold_score: 35,
        board: { tiles: [] },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockGameData,
      })

      const result = await gameApi.getDailyGame('2025-06-07')

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:3001/api/game/date/2025-06-07',
        {
          headers: {
            'Content-Type': 'application/json',
          },
        }
      )
      expect(result).toEqual(mockGameData)
    })
  })

  describe('API endpoint consistency', () => {
    it('should return same game data structure from both sequence and date endpoints', async () => {
      const gameDataFromSequence = {
        id: 'same-game-id',
        date: '2025-06-08',
        sequence_number: 1,
        threshold_score: 40,
        board: { tiles: [] },
      }

      const gameDataFromDate = { ...gameDataFromSequence }

      // Mock sequence endpoint
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => gameDataFromSequence,
      })

      const sequenceResult = await gameApi.getGameBySequence(1)

      // Mock date endpoint
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => gameDataFromDate,
      })

      const dateResult = await gameApi.getDailyGame('2025-06-08')

      // Both should return the same structure
      expect(sequenceResult).toEqual(dateResult)
      expect(sequenceResult.id).toBe(gameDataFromSequence.id)
      expect(sequenceResult.sequence_number).toBe(gameDataFromSequence.sequence_number)
    })
  })
})