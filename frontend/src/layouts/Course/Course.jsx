import Roadmap from "../../components/Roadmap";
import Item from "../../components/Item";
import { recommendedCourses } from "../../data/selfLearnDashboard.js";

function Course({ study = false }) {
  if (study) {
    return <Roadmap />;
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
      {recommendedCourses.map((course) => (
        <Item
          key={course.id}
          name={course.title}
          description={course.description}
          complete={course.progress}
          duration={course.duration}
          statusLabel={course.statusLabel}
          tone={course.tone}
        />
      ))}
    </div>
  );
}

export default Course;
