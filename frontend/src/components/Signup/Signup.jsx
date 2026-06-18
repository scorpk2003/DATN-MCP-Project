function Signup() {
    return (
        <div className="items-center justify-center flex top-0 left-0 w-full h-full bg-[rgba(0,0,0,0.3)]!">
            <div className="flex flex-col items-center justify-around bg-secondary! p-10 rounded-4xl h-130 w-97">
                <h1 className="text-5xl font-bold my-6 p-4 text-primary!">Sign up!!!</h1>
                <form className="flex flex-col items-center justify-center gap-10 text-primary!">
                    <h3>Gmail</h3>
                    <input type="text" placeholder="Gmail"  className=""/>
                    <h3>Password</h3>
                    <input type="password" placeholder="Password" />
                    <button type="submit">Signup</button>
                </form>
            </div>
        </div>
    )
}

export default Signup;