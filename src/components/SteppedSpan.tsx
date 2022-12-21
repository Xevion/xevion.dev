import {FunctionComponent} from "react";

type SteppedSpanProps = {
    children: string;
}

const SteppedSpan: FunctionComponent<SteppedSpanProps> = ({children}: SteppedSpanProps) => {
    return <div className="stepped">

    {children.split('').map((char: string) => {
        return <span>
            {char}
        </span>
    })}
    </div>
}

export default SteppedSpan;