import { useEffect, useState } from "react"
import { Board } from "./gameview/Board"
import { TileBar } from "./gameview/TileBar"
import { Header } from "./gameview/Header"
import { BoardT, BoardTileT, GameT, LetterT, MoveT, PlayerT, PositionT, TileT } from "./game-types"
import "./GameView.scss"
import { MoveHistory } from "./gameview/MoveHistory"

export type GameViewProps = {
  game: GameT
  name: string | undefined
  playMove: (move: MoveT) => void
  exchangeTiles: () => void
}

const arrAppend = <T,>(arr: Array<T>, val: T): Array<T> => [...arr, val]
const arrRemove = <T,>(arr: Array<T>, i: number): Array<T> => arr.filter((_val, idx) => idx !== i)
const applyMove = (board: BoardT, move: MoveT): BoardT => {
  const newBoard = structuredClone(board)
  for (const [[x, y], tile] of move.tiles)
    newBoard[x][y] = tile
  return newBoard
}
const moveContainsPosition = (move: MoveT, pos: PositionT): boolean => move.tiles.some(([p, _t]) => p[0] == pos[0] && p[1] == pos[1])

const getPlayer = (game: GameT, name: string | undefined): PlayerT & { index: number } | undefined => {
  const i = game.players.findIndex(p => p.name === name)
  return (i === -1) ? undefined : { ...game.players[i], index: i }
}

const tilesOfName = (game: GameT, name: string | undefined): Array<TileT> => {
  const p = getPlayer(game, name)
  return p ? structuredClone(p.tiles) : []
}

const isBlank = (t: BoardTileT | TileT): boolean => t === 'Blank' || (typeof t === 'object' && 'Blank' in t)

const tileToBoardTile = (t: TileT): BoardTileT | null => {
  const promptForBlankTileFill = (): LetterT | null => {
    const re = /^\s*[a-zA-Z]\s*$/
    let match = null;
    while (match === null) {
      const answer = prompt("What letter should the blank tile be?");
      if (answer === null)
        return null
      else
        match = answer.match(re);
    }

    return match[0].trim().toUpperCase() as LetterT
  }

  if (isBlank(t)) {
    const letter = promptForBlankTileFill()
    if (letter === null)
      return null
    return { Blank: letter }
  } else {
    return t as BoardTileT
  }
}

const boardTileToTile = (bt: BoardTileT): TileT => isBlank(bt) ? 'Blank' : bt as TileT

export const GameView = ({ game, name, playMove, exchangeTiles }: GameViewProps) => {
  const [availableTiles, setAvailableTiles] = useState<Array<TileT>>(tilesOfName(game, name))
  const [selectedTile, setSelectedTile] = useState<number | undefined>(undefined)
  const [move, setMove] = useState<MoveT>({ tiles: [] })
  const [resetMarker, setResetMarker] = useState(false)

  useEffect(() => {
    setAvailableTiles(tilesOfName(game, name))
    setSelectedTile(undefined)
    setMove({ tiles: [] })
  }, [resetMarker, game, name]);

  const onClickTileBarTile = (selection: number) =>
    setSelectedTile(selection !== selectedTile ? selection : undefined)

  const onClickResetTiles = () => setResetMarker(!resetMarker)
  const onClickSubmitMove = () => playMove(move)
  const notYourTurn = game.whose_turn !== getPlayer(game, name)?.index
  const board = applyMove(game.board, move)
  const onClickBoardSquare = (x: number, y: number, occupied: boolean) => {
    if (selectedTile !== undefined && !occupied) {
      const boardTile = tileToBoardTile(availableTiles[selectedTile]);
      if (boardTile) {
        setMove({ tiles: arrAppend(move.tiles, [[x, y], boardTile]) })
        setAvailableTiles(arrRemove(availableTiles, selectedTile))
      }
      setSelectedTile(undefined)
    } else if (occupied && moveContainsPosition(move, [x, y])) {
      const tileIndex = move.tiles.findIndex(([pos, _t]) => pos[0] == x && pos[1] == y)
      const tile = boardTileToTile(move.tiles[tileIndex][1]);
      setMove({ tiles: arrRemove(move.tiles, tileIndex) })
      setAvailableTiles(arrAppend(availableTiles, tile))
      setSelectedTile(undefined)
    }
  }
  const onClickExchangeTiles = () => {
    if (confirm("Swap all of your tiles for new ones? (skip your turn)"))
      exchangeTiles()
  }

  return (
    <div className="game-view">
      <Header game={game} name={name} />
      <div className="tile-bar-div">
        <h2 className="label">Your Tiles:</h2>
        <TileBar tiles={availableTiles} onClickTile={onClickTileBarTile} selectedTile={selectedTile} />
        <button className="button" onClick={onClickResetTiles}>Restore Tiles</button>
        <button className="button" onClick={onClickSubmitMove} disabled={notYourTurn}>Submit Move</button>
        <button className="button" onClick={onClickExchangeTiles} disabled={notYourTurn}>Exchange Tiles</button>
      </div>
      <div className="board-center">
        <Board board={board} onClickSquare={onClickBoardSquare} />
      </div>
      <MoveHistory game={game} />
    </div>
  )
}
