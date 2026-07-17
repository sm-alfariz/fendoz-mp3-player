import { PlayerProvider } from './contexts/PlayerContext';
import { PlaylistProvider } from './contexts/PlaylistContext';
import { EqProvider } from './contexts/EqContext';
import { LyricsProvider } from './contexts/LyricsContext';
import { Sidebar } from './components/Sidebar';
import { PlayerBar } from './components/PlayerBar';
import { NowPlaying } from './components/NowPlaying';
import { PlaylistView } from './components/PlaylistView';
import { FileExplorer } from './components/FileExplorer';
import './App.css';

function App() {
  return (
    <PlayerProvider>
      <PlaylistProvider>
        <EqProvider>
          <LyricsProvider>
            <div className="app">
              <div className="app-sidebar">
                <Sidebar />
                <FileExplorer />
              </div>

              <div className="app-main">
                <div className="main-tabs">
                  <NowPlaying />
                  <PlaylistView />
                </div>
              </div>

              <div className="app-player">
                <PlayerBar />
              </div>
            </div>
          </LyricsProvider>
        </EqProvider>
      </PlaylistProvider>
    </PlayerProvider>
  );
}

export default App;
