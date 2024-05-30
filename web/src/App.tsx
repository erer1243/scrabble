import './App.css'
import { Board } from './Board'
import { TileBar } from './TileBar'
import { newBoard } from './game'

const App = () => {

  return (
    <>
      <TileBar letters={["A", "B", "C", "D", "Z"]} />
      <Board board={newBoard()} />
    </>
  )
  
  // let elems: Array<JSX.Element> = [];
  // for (let l of "abcdefghijklmnopqrstuvwxyz ") {
  //   elems.push(<Tile letter={l} pointValue={10} selectable={true} selected={false} />);
  // }

  // return elems;
}

export default App
