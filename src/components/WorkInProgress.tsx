import type {FunctionComponent} from "react";

const WorkInProgress: FunctionComponent = () => {
    return <div className="w-full my-10 flex justify-center">
        <div
            className="bg-zinc-850 border border-zinc-700 rounded-md max-w-screen-md w-full m-1 p-5 flex flex-col items-center justify-center">
            <p className="font-semibold text-3xl sm:text-4xl pb-2">Work In Progress</p>
            <p className="text-lg text-center">
                This website is a work-in progress.
                <br/>
                Unfortunately, this page hasn&apos;t been finished yet. Check back later.
        </p>
        </div>
    </div>
}

export default WorkInProgress;