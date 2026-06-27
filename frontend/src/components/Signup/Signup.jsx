function Signup() {
    return (
        <div className="flex h-full w-full items-center justify-center bg-[var(--overlay-scrim)]">
            <div className="flex w-full max-w-sm flex-col items-center justify-around rounded-[var(--radius-card)] border border-[var(--border-primary)] bg-[var(--bg-surface)] p-10 shadow-[var(--shadow-card)]">
                <h1 className="my-6 p-4 text-3xl font-bold text-[var(--text-primary)]">Sign up</h1>
                <form className="flex w-full flex-col items-stretch justify-center gap-4 text-[var(--text-primary)]">
                    <h3>Gmail</h3>
                    <input type="text" placeholder="Gmail"  className="h-10 cursor-text rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3"/>
                    <h3>Password</h3>
                    <input type="password" placeholder="Password" className="h-10 cursor-text rounded-[var(--radius-md)] border border-[var(--border-secondary)] bg-[var(--bg-surface)] px-3" />
                    <button type="submit" className="h-10 cursor-pointer rounded-[var(--radius-md)] bg-[var(--action-primary)] px-4 font-semibold text-[var(--action-primary-text)] hover:bg-[var(--action-primary-hover)]">Signup</button>
                </form>
            </div>
        </div>
    )
}

export default Signup;
