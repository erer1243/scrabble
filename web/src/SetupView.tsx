import { GameT } from "./game-types"
import "./SetupView.scss"

export type SetupViewProps = {
  game: GameT
  name: string | undefined
  joinGame: (name: string) => void
  startGame: () => void
}

export const SetupView = ({ game, name, joinGame, startGame }: SetupViewProps) => {
  let joinGameArea
  if (name === undefined) {
    const onClickButton = () => {
      const input = (document.getElementById("name-input")! as HTMLInputElement).value.trim()
      if (input)
        joinGame(input)
      else
        alert("Enter a name")
    }
    joinGameArea = (
      <div className="setup-view-join-game-area">
        <input className="setup-view-name-input" id="name-input" placeholder="Name"></input>
        <button className="setup-view-join-button" onClick={onClickButton}>Join Game</button>
      </div>
    )
  }

  let playerList
  if (game.players.length == 0) {
    playerList = <h3>None yet</h3>
  } else {
    const playerListItems = game.players.map((player, i) => {
      let thisIsMeButton
      let thisIsYou
      if (name === undefined)
        thisIsMeButton = <button onClick={() => joinGame(player.name)}>This is me</button>
      else if (name === player.name)
        thisIsYou = "(you)"
      return <li key={i}>{player.name} {thisIsMeButton} {thisIsYou}</li>
    })
    playerList = <ul className="setup-view-player-list">{playerListItems}</ul>
  }

  const startGameButtonDisabled = game.players.length < 2 && false
  const startGameButton = <button onClick={startGame} disabled={startGameButtonDisabled}>Start The Game</button>
  
  return (
    <div className="setup-view">
      <h1>Game setup</h1>
      {joinGameArea}
      <br/>
      {startGameButton}
      <h2>Players:</h2>
      {playerList}
    </div>
  )
}
