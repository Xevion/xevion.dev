import {type NextPage} from "next";
import AppWrapper from "../components/AppWrapper";
import WorkInProgress from "../components/WorkInProgress";

const ContactPage: NextPage = () => {
    return <AppWrapper current='contact'>
        <WorkInProgress/>
    </AppWrapper>
}

export default ContactPage;