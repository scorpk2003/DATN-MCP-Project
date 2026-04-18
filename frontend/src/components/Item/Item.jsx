function Item({name, description, complete, bgcolor = `#2a2529`}) {
    return (
        <div className="h-45 w-38 rounded-lg **:text-primary flex flex-col gap-2 p-3 **:hover:cursor-pointer **:bg-transparent hover:-translate-y-3 transform hover:duration-400 duration-1000" style={{backgroundColor: `${bgcolor}CC`}}>
            <div className="text-sm font-bold p-2 truncate">{ name }</div>
            <div className="text-xs p-1.5 flex-1 overflow-y-auto">{ description }</div>
            <div className="h-1.5 w-full bg-white/20! rounded-full mt-auto flex flex-row">
                <div className="h-full bg-white! rounded-full" style={ { width: `${complete * 100}%` } }></div>
            </div>
        
        </div>
    );
}

export default Item;