function Sidebar() {
    return (
        <div className="min-h-screen p-10 bg-txt rounded-r-lg max-w-sm w-55 items-center flex flex-col gap-5">
            <div className="bg-transparent text-primary font-bold text-2xl">Infomation</div>
            <nav className="p-4 my-3">
                <div>List Chat01</div>
                <div>List Chat02</div>
            </nav>
        </div>
    );
}

export default Sidebar;