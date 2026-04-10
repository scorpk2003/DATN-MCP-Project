import { useState } from "react";
import Signup from "../../components/Signup";
function Header() {
    const [isSignup, setIsSignup] = useState(false);
    return (
        <div className="bg-primary h-85 w-full p-5 flex flex-col **:bg-transparent">
            <div className="flex flex-row justify-around m-3 py-3">
                <nav className="gap-10 font-bold flex flex-row justify-around px-2 py-3">
                    <div>Home</div>
                    <div>Roadmap</div>
                    <div>Course</div>
                </nav>
                <div className="basis-1/3 items-center"><h1>Logo</h1></div>
                <div className="gap-5 flex flex-row mx-3 items-center px-2">
                    <div>Sign In</div>
                    <div className="!bg-secondary text-white rounded-full px-3" onClick={() => setIsSignup(true)}>Sign Up</div>
                </div>
            </div>
            <div className="items-center justify-center flex text-4xl font-bold basis-2/3">
                Planning
            </div>
            {isSignup &&
            <div className="absolute top-0 left-0 w-full h-full">
                <Signup />
            </div>
            }
        </div>
    );
}

export default Header;