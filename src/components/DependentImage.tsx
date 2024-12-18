import Image, { ImageProps } from "next/image";
import { useMemo, useState } from "react";

type DependentProps = {
  className?: string | ((loaded: boolean) => string);
};

type DependentImageProps = Omit<ImageProps, "className"> & DependentProps;

const DependentImage = (props: DependentImageProps) => {
  const [loaded, setLoaded] = useState(false);
  const { className } = props;
  const renderedClassName = useMemo(() => {
    if (className === undefined) return "";
    if (typeof className === "function") return className(loaded);
    return className;
  }, [loaded, className]);

  return (
    <Image
      {...props}
      className={renderedClassName}
      alt="no"
      onLoadingComplete={() => {
        setLoaded(true);
      }}
    />
  );
};

export default DependentImage;
