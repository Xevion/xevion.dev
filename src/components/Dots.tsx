import React, { useEffect, useRef } from 'react';
import { p5i, P5I } from 'p5i';

const Dots: React.FC = () => {
    const canvasRef = useRef<HTMLDivElement | null>(null);

    const {
        mount,
        unmount,
        createCanvas,
        background,
        noFill,
        stroke,
        noise,
        noiseSeed,
        resizeCanvas,
        cos,
        sin,
        TWO_PI,
    } = p5i();

    let w = window.innerWidth;
    let h = window.innerHeight;
    const offsetY = window.scrollY;

    const SCALE = 200;
    const LENGTH = 10;
    const SPACING = 15;

    function getForceOnPoint(x: number, y: number, z: number) {
        return (noise(x / SCALE, y / SCALE, z) - 0.5) * 2 * TWO_PI;
    }

    const existingPoints = new Set<string>();
    const points: { x: number, y: number, opacity: number }[] = [];

    function addPoints() {
        for (let x = -SPACING / 2; x < w + SPACING; x += SPACING) {
            for (let y = -SPACING / 2; y < h + offsetY + SPACING; y += SPACING) {
                const id = `${x}-${y}`;
                if (existingPoints.has(id)) continue;
                existingPoints.add(id);
                points.push({ x, y, opacity: Math.random() * 0.5 + 0.5 });
            }
        }
    }

    function setup() {
        createCanvas(w, h);
        background('#000000');
        stroke('rgba(170, 170, 170, 0.05)');
        noFill();

        noiseSeed(+new Date());

        addPoints();


    }

    function draw({ circle }: P5I) {
        background('#000000');
        const t = +new Date() / 10000;

        for (const p of points) {
            const { x, y } = p;
            const rad = getForceOnPoint(x, y, t);
            const length = (noise(x / SCALE, y / SCALE, t * 2) + 0.5) * LENGTH;
            const nx = x + cos(rad) * length;
            const ny = y + sin(rad) * length;

            const opacity = 1;
            
            // const center_distance = Math.sqrt((x - w / 2) ** 2 + (y - h / 2) ** 2);
            // if (center_distance < 350)
            //     opacity = 0;

            //     opacity = 
            stroke(200, 200, 200, (Math.abs(cos(rad)) * 0.8 + 0.2) * p.opacity * 255 * 0.5 * opacity);
            circle(nx, ny - offsetY, 1);
        }
    }

    function restart() {
        if (canvasRef.current) {
            mount(canvasRef.current, { setup, draw });
        }
    }

    useEffect(() => {
        restart();

        const handleResize = () => {
            w = window.innerWidth;
            h = window.innerHeight;
            resizeCanvas(w, h);
            addPoints();
        };

        window.addEventListener('resize', handleResize);

        return () => {
            window.removeEventListener('resize', handleResize);
            unmount();
        };
    }, []);

    return <div ref={canvasRef} className='fixed left-0 right-0 top-0 bottom-0 pointer-events-none -z-1 animate-bg delay-500' />;
};

export default Dots;