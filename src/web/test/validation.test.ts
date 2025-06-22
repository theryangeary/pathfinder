import { describe, expect, it } from "vitest";
import { validateAllAnswers } from "../utils/validation";
import { testBoard } from "./util.test";

describe('test validation', () => {
    it('should say silo, seed, sed, sold, does is a valid set', () => {
        const board = testBoard('hissc*lole*dseeo');
        const validation = validateAllAnswers(
            board,
            ['silo', 'seed', 'sed', 'sold', 'does'],
            true,
            ((_: string) => true),
        );
        expect(validation.validAnswers.filter((b => !b)).length == 0).toBeTruthy
    });
})
