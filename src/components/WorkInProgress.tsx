import type {FunctionComponent} from "react";

const WorkInProgress: FunctionComponent = () => {
    return <div className="w-full my-10 flex flex-col items-center">
        <div
            className="bg-zinc-800 border border-zinc-700 rounded-md max-w-[23rem] sm:max-w-[25rem] lg:max-w-[30rem] mx-3 w-full p-5 flex flex-col items-center justify-center">
            <span className="bg-gradient-to-r from-orange-500 via-fuchsia-600 to-cyan-500 text-transparent font-semibold bg-clip-text text-3xl sm:text-4xl pb-2 text-center">
                Work In Progress
            </span>
            <p className="text-lg text-center">
                This website is a work in-progress.
                <br/>
                Unfortunately, this page hasn&apos;t been finished yet. Check back later.
            </p>
        </div>
    </div>
}

export default WorkInProgress;