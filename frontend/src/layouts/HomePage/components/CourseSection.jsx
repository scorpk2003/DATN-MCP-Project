import { Button, SectionTitle } from "../../../components/ui";
import { CourseCard } from "./CourseCard.jsx";

export function CourseSection({ courses }) {
  return (
    <section className="space-y-4">
      <SectionTitle
        eyebrow="Learning path"
        title="Khoá học nên tiếp tục"
        action={<Button variant="ghost">Xem tất cả</Button>}
      />
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {courses.map((course) => (
          <CourseCard key={course.id} course={course} />
        ))}
      </div>
    </section>
  );
}
