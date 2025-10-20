import './Logo.css';

const PathfinderLogo = () => {
  return (
    <a href="/" style={{
      color: 'inherit', /* no blue text */
      textDecoration: 'inherit', /* no underline */
    }}>
    <div className="logo-container">
        <h1 className="logo-text">
            <span className="pathfinder">PATHFINDER</span>
            <span className="dot">.</span><span className="prof">prof</span>
        </h1>
    </div>
    </a>
  );
};

export default PathfinderLogo;
