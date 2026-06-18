import AppShell from "./layouts/AppShell";
import HomePage from "./layouts/HomePage";
import {Route, Routes} from "react-router-dom";

function App() {

  return (
    <Routes>
      <Route element={ <AppShell /> }>
        <Route path="/" element={ <HomePage /> } />
      </Route>
    </Routes>
  )
}

export default App
