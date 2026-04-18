import MainPage from "../MainPage";
import Sidebar from "../Sidebar";
function AppShell() {
    return ( 
        <div className="flex flex-row h-screen w-screen">
            <Sidebar />
            <MainPage />
        </div>
    );
}

export default AppShell;