// import { Protocol } from "@popup/components/Protocol";
import ReactDOM from "react-dom";
function App(id: string) {
    ReactDOM.render(
        <h1>Hello, world!</h1>,
        document.getElementById(id)
      );
//   return (
//     <Protocol>
//     </Protocol>
//   );
}

export default App;
