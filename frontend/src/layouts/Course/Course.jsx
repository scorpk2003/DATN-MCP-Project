import { useState } from "react";
import Roadmap from "../../components/Roadmap";
import Item from "../../components/Item";

const listcourse = [
    {
        id: 1,
        name: "Course 01xxxxxxxxxxxxxxxxxxxxxxxxxx",
        description: "Description of course 01",
        complete: 0.5,
    },
    {
        id: 2,
        name: "Course 02",
        description: "Description of course 02",
        complete: 0.2,
    }
]

function Course() {
    let [study, SetStudy] = useState(false);
    return (
        <div className="m-10">
            { study ? <Roadmap /> :
                <div className="grid-cols-3 grid gap-7">
                    {listcourse.map((course) => (
                        <Item
                            key={course.id}
                            name={course.name}
                            description={course.description}
                            complete={course.complete}
                        />
                    ))}
                </div>
            }
        </div>
    );
}

export default Course;