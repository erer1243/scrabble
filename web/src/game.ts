export type OptionT<T> = T | null;
export type TileT = 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M'
  | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T' | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | 'Blank'
export type ModifierT = "DoubleLetter" | "TripleLetter" | "DoubleWord" | "TripleWord"
export type DirectionT = "Right" | "Down"
export type PointT = [number, number];
export type BoardT = Array<Array<OptionT<TileT>>> // 15x15
export type InvalidMoveReasonT = "Disconnected" | "NotAWord" | { "Impossible": string }
export type MoveT = {
  letters: Array<TileT>,
  start: PointT,
  direction: DirectionT,
}
export type PlayerT = {
  tiles: Array<TileT>,
  score: number,
  name: string,
}
export type GameT = {
  board: BoardT,
  tile_bag: Array<TileT>,
  players: Array<PlayerT>,
  whose_turn: number,
}

// Constants from the game
export const tileValues: Record<TileT, number> = {
  'A': 1, 'E': 1, 'I': 1, 'O': 1, 'U': 1, 'L': 1, 'N': 1, 'S': 1, 'T': 1, 'R': 1,
  'D': 2, 'G': 2, 'B': 3, 'C': 3, 'M': 3, 'P': 3, 'F': 4, 'H': 4, 'V': 4, 'W': 4,
  'Y': 4, 'K': 5, 'J': 8, 'X': 8, 'Q': 10, 'Z': 10, 'Blank': 0,
}

export const modifiers: Array<Array<ModifierT | undefined>> = (() => {
  const modifierMap: Record<string, ModifierT> = {
    "7,7": "DoubleLetter",
    "2,8": "DoubleLetter",
    "0,0": "TripleWord",
    "6,2": "DoubleLetter",
    "7,11": "DoubleLetter",
    "0,14": "TripleWord",
    "5,1": "TripleLetter",
    "13,13": "DoubleWord",
    "5,5": "TripleLetter",
    "13,9": "TripleLetter",
    "2,6": "DoubleLetter",
    "11,11": "DoubleWord",
    "3,7": "DoubleLetter",
    "12,8": "DoubleLetter",
    "10,10": "DoubleWord",
    "4,10": "DoubleWord",
    "6,6": "DoubleLetter",
    "10,4": "DoubleWord",
    "0,3": "DoubleLetter",
    "2,12": "DoubleWord",
    "6,12": "DoubleLetter",
    "7,14": "TripleWord",
    "9,13": "TripleLetter",
    "11,14": "DoubleLetter",
    "12,2": "DoubleWord",
    "14,14": "TripleWord",
    "7,3": "DoubleLetter",
    "8,2": "DoubleLetter",
    "3,14": "DoubleLetter",
    "14,0": "TripleWord",
    "13,5": "TripleLetter",
    "5,9": "TripleLetter",
    "9,1": "TripleLetter",
    "3,3": "DoubleWord",
    "8,6": "DoubleLetter",
    "1,5": "TripleLetter",
    "6,8": "DoubleLetter",
    "2,2": "DoubleWord",
    "5,13": "TripleLetter",
    "9,5": "TripleLetter",
    "8,8": "DoubleLetter",
    "14,3": "DoubleLetter",
    "11,7": "DoubleLetter",
    "9,9": "TripleLetter",
    "1,13": "DoubleWord",
    "0,7": "TripleWord",
    "0,11": "DoubleLetter",
    "12,6": "DoubleLetter",
    "14,7": "TripleWord",
    "11,3": "DoubleWord",
    "4,4": "DoubleWord",
    "11,0": "DoubleLetter",
    "8,12": "DoubleLetter",
    "13,1": "DoubleWord",
    "3,11": "DoubleWord",
    "14,11": "DoubleLetter",
    "1,9": "TripleLetter",
    "3,0": "DoubleLetter",
    "1,1": "DoubleWord",
    "7,0": "TripleWord",
    "12,12": "DoubleWord"
  }

  const board: Array<Array<ModifierT | undefined>> = [];
  for (let x = 0; x < 15; x++) {
    board[x] = [];
    for (let y = 0; y < 15; y++) {
      board[x][y] = modifierMap[`${x},${y}`];
    }
  }
  return board;
})()
