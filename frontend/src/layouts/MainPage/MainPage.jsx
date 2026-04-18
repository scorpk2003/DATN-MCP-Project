import { useState } from "react";
import Search from "../Search";

function MainPage() {
    let [isLogin, SetLogin] = useState(false);
    return (<div className="h-full m-4">
        { !isLogin &&
            <div className="absolute flex flex-row gap-9 w-1/7 right-0 my-15 **:hover:cursor-pointer">
                <div className="py-1.5">Login</div>
                <div className="bg-txt text-primary rounded-full px-5 py-1.5">Sign up</div>
            </div>
        }
        <Search />
    </div>);
}

export default MainPage;