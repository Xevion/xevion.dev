import {type NextPage} from "next";
import AppWrapper from "../components/AppWrapper";
import {RiMagicLine} from "react-icons/ri";
import {BiBus, BiHash, BiNetworkChart} from "react-icons/bi";
import Link from "next/link";
import {IconType} from "react-icons";
import {HiOutlineRss} from "react-icons/hi";
import {GiHummingbird, GiPathDistance} from "react-icons/gi";
import {MdOutlineGrain, MdOutlineDns, MdOutlineLeaderboard} from "react-icons/md";
import {FiLayers} from "react-icons/fi";
import {FaReact} from "react-icons/fa";
import {SiPowershell} from "react-icons/si";

const ProjectsPage: NextPage = () => {
    const projects: { name: string, description: string, url?: string, icon: IconType }[] = [
        {
            name: "Portal",
            description: "ACM Membership & Event System",
            url: "https://portal.acmutsa.org",
            icon: RiMagicLine
        },
        {
            name: "Runnerspace",
            description: "Hackathon MySpace Clone",
            url: "https://runnerspace.xevion.dev",
            icon: RiMagicLine
        },
        {
            name: "v6 Place",
            description: "Paint Images with IPv6 Addresses",
            url: "https://github.com/Xevion/v6-place",
            icon: BiNetworkChart
        },
        {
            name: "Phototag",
            description: "Effortlessly Tag Photos",
            url: "/phototag",
            icon: BiHash
        },
        {
            name: "Paths",
            description: "Graph Traversal Algorithms",
            url: "https://github.com/Xevion/Paths",
            icon: GiPathDistance
        },
        {
            name: "v2.xevion.dev",
            description: "Jekyll-based Blog",
            url: "https://v2.xevion.dev",
            icon: HiOutlineRss
        },
        {
            name: "Grain",
            description: "Pretty SVG-based Noise",
            url: "https://grain.xevion.dev",
            icon: MdOutlineGrain
        },
        {
            name: "The Office",
            description: "View Quotes from The Office",
            url: "https://the-office.xevion.dev",
            icon: FiLayers
        },
        {
            name: "Boids",
            description: "Flocking Simulation",
            url: "https://github.com/Xevion/Boids",
            icon: GiHummingbird
        },
        {
            name: "bus-reminder",
            description: "Last Bus Departure Reminder",
            url: "http://github.com/Xevion/bus-reminder",
            icon: BiBus
        },
        {
            name: "rdap",
            description: "Javascript RDAP Client",
            url: "https://rdap.xevion.dev",
            icon: MdOutlineDns
        },
        {
            name: "icons",
            description: "Dynamic React-Icons Loading",
            url: "https://icons.xevion.dev",
            icon: FaReact
        },
        {
            name: "trivia",
            description: "Trivia Leaderboard in Flask",
            url: "http://github.com/Xevion/trivia",
            icon: MdOutlineLeaderboard
        },
        {
            name: "Powershell",
            description: "Scripts & Guides",
            url: "https://powershell.xevion.dev",
            icon: SiPowershell
        }
    ]
    return <AppWrapper current='projects'>
        <div className="mt-20 grid grid-cols-3 gap-x-20 gap-y-14 h-full py-2 max-w-500 w-max mx-auto flex ">
            {projects.map(({name, description, url, icon: Icon}) => <Link
                key={name}
                className="relative flex flex-shrink items-center opacity-75 hover:opacity-100 transition-opacity"
                href={url ?? ""}
                target="_blank"
                rel="noreferrer"
                title={name}>
                <div className="pt-2 pr-5">
                    <Icon className="text-3xl saturate-0 opacity-80"/>
                </div>
                <div className="flex-auto">
                    <div className="text-normal">{name}</div>
                    <div className="text-sm opacity-70 font-normal">{description}</div>
                </div>
            </Link>)}
        </div>
    </AppWrapper>
};

export default ProjectsPage;