// Repositories at this level are responsible for data access and manipulation between multiple entities
// Instead of the lower level repositories where each entity has its own repository to control himself
// This is a higher level repository that can control multiple entities to make a cohesive and workable business logic

pub mod account;
pub mod room;