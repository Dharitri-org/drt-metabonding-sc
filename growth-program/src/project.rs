dharitri_sc::imports!();

pub type ProjectId = u32;

#[dharitri_sc::module]
pub trait ProjectsModule {
    #[only_owner]
    #[endpoint(addProject)]
    fn add_project(&self, project_owner: ManagedAddress) -> ProjectId {
        let project_id = self.get_and_save_new_project_id();
        self.project_owner(project_id).set(project_owner);

        project_id
    }

    #[only_owner]
    #[endpoint(setProjectOwner)]
    fn set_project_owner(&self, project_id: ProjectId, new_owner: ManagedAddress) {
        self.project_owner(project_id).set(new_owner);
    }

    fn get_and_save_new_project_id(&self) -> ProjectId {
        let new_project_id = self.last_project_id().get() + 1;
        self.last_project_id().set(new_project_id);

        new_project_id
    }

    fn require_is_project_owner(&self, address: &ManagedAddress, project_id: ProjectId) {
        let project_owner = self.project_owner(project_id).get();
        require!(
            address == &project_owner,
            "Only project owner may call this endpoint"
        );
    }

    fn require_valid_project_id(&self, project_id: ProjectId) {
        let last_project_id = self.last_project_id().get();
        require!(project_id <= last_project_id, "Invalid project ID");
    }

    #[storage_mapper("lastProjectId")]
    fn last_project_id(&self) -> SingleValueMapper<ProjectId>;

    #[storage_mapper("projectOwner")]
    fn project_owner(&self, project_id: ProjectId) -> SingleValueMapper<ManagedAddress>;
}
