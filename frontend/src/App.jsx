import MainPage from "./layouts/MainPage"
import Sidebar from "./layouts/Sidebar"

function App() {

  return (
    <div className="flex flex-row h-screen w-screen">
      <Sidebar />
      <MainPage />
    </div>
  )
}

export default App
