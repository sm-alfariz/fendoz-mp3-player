import { useRef, useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { usePlayer } from '../contexts/PlayerContext';

type VisualizerMode = 'bars' | 'circular' | 'oscilloscope';

export function Visualizer() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [mode, setMode] = useState<VisualizerMode>('bars');
  const { state } = usePlayer();
  const animationRef = useRef<number | undefined>(undefined);
  const fftDataRef = useRef<number[]>(new Array(64).fill(0));

  // Listen for FFT data from Rust
  useEffect(() => {
    const unlisten = listen<{ frequencies: number[] }>('fft-data', (event) => {
      fftDataRef.current = event.payload.frequencies;
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Animation loop
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const draw = () => {
      const width = canvas.width;
      const height = canvas.height;

      // Clear canvas
      ctx.fillStyle = 'rgba(0, 0, 0, 0.1)';
      ctx.fillRect(0, 0, width, height);

      const fftData = fftDataRef.current;

      switch (mode) {
        case 'bars':
          drawBars(ctx, fftData, width, height);
          break;
        case 'circular':
          drawCircular(ctx, fftData, width, height);
          break;
        case 'oscilloscope':
          drawOscilloscope(ctx, fftData, width, height);
          break;
      }

      animationRef.current = requestAnimationFrame(draw);
    };

    if (state === 'playing') {
      draw();
    }

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [mode, state]);

  const drawBars = (
    ctx: CanvasRenderingContext2D,
    data: number[],
    width: number,
    height: number
  ) => {
    const barCount = Math.min(data.length, 32);
    const barWidth = (width / barCount) - 2;
    const gap = 2;

    for (let i = 0; i < barCount; i++) {
      const value = data[i] || 0;
      const barHeight = value * height * 2;

      const x = i * (barWidth + gap);
      const y = height - barHeight;

      // Gradient color based on frequency
      const hue = (i / barCount) * 60 + 200; // Blue to purple
      ctx.fillStyle = `hsl(${hue}, 80%, 50%)`;
      ctx.fillRect(x, y, barWidth, barHeight);
    }
  };

  const drawCircular = (
    ctx: CanvasRenderingContext2D,
    data: number[],
    width: number,
    height: number
  ) => {
    const centerX = width / 2;
    const centerY = height / 2;
    const radius = Math.min(width, height) / 4;
    const barCount = Math.min(data.length, 64);

    for (let i = 0; i < barCount; i++) {
      const value = data[i] || 0;
      const angle = (i / barCount) * Math.PI * 2;
      const barLength = value * radius * 2;

      const x1 = centerX + Math.cos(angle) * radius;
      const y1 = centerY + Math.sin(angle) * radius;
      const x2 = centerX + Math.cos(angle) * (radius + barLength);
      const y2 = centerY + Math.sin(angle) * (radius + barLength);

      const hue = (i / barCount) * 360;
      ctx.strokeStyle = `hsl(${hue}, 80%, 50%)`;
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(x1, y1);
      ctx.lineTo(x2, y2);
      ctx.stroke();
    }
  };

  const drawOscilloscope = (
    ctx: CanvasRenderingContext2D,
    data: number[],
    width: number,
    height: number
  ) => {
    const centerY = height / 2;
    const sliceWidth = width / data.length;

    ctx.strokeStyle = '#00ff00';
    ctx.lineWidth = 2;
    ctx.beginPath();

    for (let i = 0; i < data.length; i++) {
      const value = data[i] || 0;
      const y = centerY + (value - 0.5) * height;
      const x = i * sliceWidth;

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }

    ctx.stroke();
  };

  const cycleMode = () => {
    const modes: VisualizerMode[] = ['bars', 'circular', 'oscilloscope'];
    const currentIndex = modes.indexOf(mode);
    const nextIndex = (currentIndex + 1) % modes.length;
    setMode(modes[nextIndex]);
  };

  const getModeLabel = () => {
    switch (mode) {
      case 'bars':
        return 'Spectrum';
      case 'circular':
        return 'Circular';
      case 'oscilloscope':
        return 'Oscilloscope';
    }
  };

  return (
    <div className="visualizer">
      <div className="visualizer-header">
        <h3>Visualizer</h3>
        <button className="mode-btn" onClick={cycleMode}>
          {getModeLabel()}
        </button>
      </div>
      <canvas
        ref={canvasRef}
        width={400}
        height={200}
        className="visualizer-canvas"
      />
    </div>
  );
}
