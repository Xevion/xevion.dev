import type { FunctionComponent } from "react";

const WorkInProgress: FunctionComponent = () => {
  return (
    <div className="my-10 flex w-full flex-col items-center">
      <div className="mx-3 flex w-full max-w-[23rem] flex-col items-center justify-center rounded-md border border-zinc-700 bg-zinc-800/30 p-5 sm:max-w-[25rem] lg:max-w-[30rem]">
        <span className="bg-gradient-to-r from-orange-500 via-fuchsia-600 to-cyan-500 bg-clip-text pb-2 text-center text-3xl font-semibold text-transparent sm:text-4xl">
          Work In Progress
        </span>
        <p className="text-center text-lg">
          This website is a work in-progress.
          <br />
          Unfortunately, this page hasn&apos;t been finished yet. Check back
          later.
        </p>
      </div>
    </div>
  );
};

export default WorkInProgress;
