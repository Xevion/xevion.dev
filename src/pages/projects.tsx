import {type NextPage} from "next";
import AppWrapper from "../components/AppWrapper";
import WorkInProgress from "../components/WorkInProgress";

const ProjectsPage: NextPage = () => {
    return <AppWrapper current='projects'>
        <WorkInProgress/>
    </AppWrapper>;
}

export default ProjectsPage;