import { Outlet } from "react-router-dom";

function MainPage() {
  return (
    <main className="min-w-0 px-4 py-5 sm:px-6 lg:px-8">
      <Outlet />
    </main>
  );
}

export default MainPage;
