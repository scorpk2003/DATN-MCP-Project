import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faEllipsisVertical } from "@fortawesome/free-solid-svg-icons";
const listchat = [
    {
        id: 1,
        des: "List Chat 01xxxxxxxxxxxxxxxxxx",
    },
    {
        id: 2,
        des: "List Chat 02",
    },
    {
        id: 3,
        des: "List Chat 03",
    },
    {
        id: 4,
        des: "List Chat 04",
    },
    {
        id: 5,
        des: "List Chat 05",
    },
    {
        id: 6,
        des: "List Chat 06",
    },
    {
        id: 7,
        des: "List Chat 07",
    },
    {
        id: 8,
        des: "List Chat 08",
    },
    {
        id: 9,
        des: "List Chat 09",
    },
    {
        id: 10,
        des: "List Chat 10",
    },
    {
        id: 11,
        des: "List Chat 11",
    },
    {
        id: 12,
        des: "List Chat 12",
    }
]
function Sidebar() {
    const handleClick = () => {
        console.log("Clicked");
    }
    return (
        <div className="p-10 bg-txt rounded-r-lg max-w-sm w-55 items-center flex flex-col gap-5 **:bg-transparent">
            <div className="basis-2/5 items-center flex">
                <p className="text-primary font-bold text-2xl">Infomation</p>
            </div>
            <nav className="p-2 my-3 h-fullmax-w-sm overflow-y-auto scroll-smooth flex flex-col gap-2 bg-transparent rounded-lg scroll-my-2">
                {listchat.map((chat) => (
                    <div key={ chat.id } className="h-7.5 flex flex-row items-center w-40 p-2 rounded-lg bg-primary/10 cursor-pointer hover:bg-primary/20 transition-colors duration-200">
                        <p className="truncate text-sm text-primary max-w-30 items-center basis-5/6">{ chat.des }</p>
                        <FontAwesomeIcon icon={faEllipsisVertical}  className="right-0 basis-1/6" onClick={handleClick}/>
                    </div>
                ))}
            </nav>
        </div>
    );
}

export default Sidebar;