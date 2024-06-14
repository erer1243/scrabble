import { useEffect, useState } from "react"
import { Board } from "./Board"
import { TileBar } from "./TileBar"
import { BoardT, GameT, MoveT, PlayerT, TileT } from "./game-types"
import "./GameView.scss"

export type GameViewProps = {
  game: GameT
  name: string | undefined
  playMove: (move: MoveT) => void
}

const arrAppend = <T,>(arr: Array<T>, val: T): Array<T> => [...arr, val]
const arrRemove = <T,>(arr: Array<T>, i: number): Array<T> => arr.filter((_val, idx) => idx !== i)
const applyMove = (board: BoardT, move: MoveT): BoardT => {
  const newBoard = structuredClone(board)
  for (const [[x, y], tile] of move.tiles)
    newBoard[x][y] = tile
  return newBoard
}

const getPlayer = (game: GameT, name: string | undefined): PlayerT & { index: number } | undefined => {
  const i = game.players.findIndex(p => p.name === name)
  return (i === -1) ? undefined : { ...game.players[i], index: i }
}

const tilesOfName = (game: GameT, name: string | undefined): Array<TileT> => {
  const p = getPlayer(game, name)
  return p ? structuredClone(p.tiles) : []
}

const scoreOfPlayer = (p: PlayerT): number => p.moves.reduce((score, move) => score + move.value, 0)

export const GameView = ({ game, name, playMove }: GameViewProps) => {
  const [availableTiles, setAvailableTiles] = useState<Array<TileT>>(tilesOfName(game, name))
  const [selectedTile, setSelectedTile] = useState<number | undefined>(undefined)
  const [move, setMove] = useState<MoveT>({ tiles: [] })
  const [resetMarker, setResetMarker] = useState(false)

  useEffect(() => {
    setAvailableTiles(tilesOfName(game, name))
    setSelectedTile(undefined)
    setMove({ tiles: [] })
  }, [resetMarker, game, name]);

  const onClickResetTiles = () => setResetMarker(!resetMarker)
  const onClickSubmitMove = () => playMove(move)
  const disableSubmit = game.whose_turn !== getPlayer(game, name)?.index
  const board = applyMove(game.board, move)
  const onClickBoardSquare = (x: number, y: number, isFilled: boolean) => {
    if (selectedTile !== undefined && !isFilled) {
      setMove({ tiles: arrAppend(move.tiles, [[x, y], availableTiles[selectedTile]]) })
      setAvailableTiles(arrRemove(availableTiles, selectedTile))
      setSelectedTile(undefined)
    }
  }

  return (
    <div className="game-view">
      <GameViewHeader game={game} name={name} />
      <div className="tile-bar-div">
        <h2 className="label">Your Tiles:</h2>
        <TileBar tiles={availableTiles} onClickTile={setSelectedTile} selectedTile={selectedTile} />
        <button className="button" onClick={onClickResetTiles}>Reset tiles</button>
        <button className="button" onClick={onClickSubmitMove} disabled={disableSubmit}>Submit Move</button>
      </div>
      <Board board={board} onClickSquare={onClickBoardSquare} />
    </div>
  )
}

const GameViewHeader = ({ game, name }: Pick<GameViewProps, "game" | "name">) => {
  const whoseTurnPlayer = game.players[game.whose_turn]
  const whoseTurn = <h3 className="whose-turn">It's {whoseTurnPlayer.name == name ? "your" : `${whoseTurnPlayer.name}'s`} turn</h3>

  const scores = game.players.map((p, i) =>
    <div className="row" key={i}>
      <p className="name">{p.name}</p>
      <p className="score">{scoreOfPlayer(p)}</p>
    </div>
  )

  return (
    <div className="header">
      <div className="scores">
        <div className="row"><p className="name">Player</p><p className="score">Score</p></div>
        {scores}
      </div>
      {whoseTurn}
    </div>
  )
}

