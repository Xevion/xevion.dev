import {FunctionComponent} from "react";

type SteppedSpanProps = {
    children: string;
}

const SteppedSpan: FunctionComponent<SteppedSpanProps> = ({children}: SteppedSpanProps) => {
    return <div className="stepped">

    {children.split('').map((char: string, index) => {
        return <span key={index}>
            {char}
        </span>
    })}
    </div>
}

export default SteppedSpan;