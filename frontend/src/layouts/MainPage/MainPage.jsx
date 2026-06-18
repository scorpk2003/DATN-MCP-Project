// import { useState } from "react";
import Search from "../Search";
import Course from "../Course";
import { Outlet } from "react-router-dom";

function MainPage() {
    return (<div className="m-4 flex-1 relative">
        <Outlet />
        <Search />
    </div>);
}

export default MainPage;