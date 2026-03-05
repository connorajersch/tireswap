import { NavLink, Outlet } from "react-router-dom";

const AppLayout = () => {
  return (
    <div className="app-shell">
      <header className="app-header">
        <div className="brand">TireSwap</div>
        <nav className="app-nav">
          <NavLink to="/" end>
            Search
          </NavLink>
          <NavLink to="/results">Results</NavLink>
        </nav>
      </header>
      <main className="app-main">
        <Outlet />
      </main>
      <footer className="app-footer">
        <span>Data-driven tire swap recommendations.</span>
      </footer>
    </div>
  );
};

export default AppLayout;
