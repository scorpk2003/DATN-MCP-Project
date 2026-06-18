import { useMatch } from "react-router-dom";
import RcmBeginCourse from "./RcmBeginCourse";
function Sidebar() {
    const isHomePage = useMatch("/");
    
    if (isHomePage) return (<RcmBeginCourse />)
    return (<div>Not Found</div>)
}

export default Sidebar;