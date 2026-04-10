import { useState } from "react";
import Search from "../Search";

function MainPage() {
    let [isLogin, SetLogin] = useState(false);
    return (<div className="h-full m-5">
        { !isLogin &&
            <div className="absolute flex flex-row gap-7 w-1/5 right-0 my-5">
                <div>Login</div>
                <div>Sign up</div>
            </div>
        }
        <Search />
    </div>);
}

export default MainPage;