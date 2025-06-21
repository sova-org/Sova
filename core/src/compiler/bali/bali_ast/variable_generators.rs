use crate::lang::variable::Variable;
use std::cmp::min;

#[derive(Debug)]
pub struct AltVariableGenerator {
    pub current_variable_number: i64,
    pub alt_variable_base_name: String,
}

impl AltVariableGenerator {
    pub fn new(alt_variable_base_name: String) -> AltVariableGenerator {
        AltVariableGenerator {
            current_variable_number: 0,
            alt_variable_base_name,
        }
    }

    pub fn get_variable(&mut self) -> Variable {
        let new_alt_variable_name =
            self.alt_variable_base_name.clone() + "_" + &self.current_variable_number.to_string();

        self.current_variable_number += 1;

        Variable::Frame(new_alt_variable_name)
    }

    pub fn get_variables_and_num(&mut self) -> (Variable, Variable, i64) {
        let num = self.current_variable_number;
        let new_alt_variable_name =
            self.alt_variable_base_name.clone() + "_" + &self.current_variable_number.to_string();

        self.current_variable_number += 1;

        (
            Variable::Frame(new_alt_variable_name.clone()),
            Variable::Instance(new_alt_variable_name),
            num,
        )
    }

    pub fn get_num_variables(&self) -> i64 {
        self.current_variable_number
    }
}

#[derive(Debug)]
pub struct LocalChoiceVariableGenerator {
    pub current_variable_number: i64,
    pub choice_variable_base_name: String,
}

impl LocalChoiceVariableGenerator {
    pub fn new(choice_variable_base_name: String) -> LocalChoiceVariableGenerator {
        LocalChoiceVariableGenerator {
            current_variable_number: 0,
            choice_variable_base_name,
        }
    }

    pub fn get_variable(&mut self) -> Variable {
        let new_choice_variable_name = self.choice_variable_base_name.clone()
            + "_"
            + &self.current_variable_number.to_string();

        self.current_variable_number += 1;

        Variable::Instance(new_choice_variable_name)
    }

    pub fn get_variable_and_number(&mut self) -> (Variable, i64) {
        let number = self.current_variable_number;
        let variable = self.get_variable();

        (variable, number)
    }

    pub fn get_num_variables(&self) -> i64 {
        self.current_variable_number
    }
}

#[derive(Debug)]
pub struct ChoiceVariableGenerator {
    pub current_variable_number: i64,
    pub choice_variable_base_name: String,
    pub target_variable_base_name: String,
    pub variable_set: Vec<Variable>,
    pub variable_bounds: Vec<i64>,
}

impl ChoiceVariableGenerator {
    pub fn new(
        choice_variable_base_name: String,
        target_variable_base_name: String,
    ) -> ChoiceVariableGenerator {
        ChoiceVariableGenerator {
            current_variable_number: 0,
            choice_variable_base_name,
            target_variable_base_name,
            variable_set: Vec::new(),
            variable_bounds: Vec::new(), // gives the bound of each variable for random generation
        }
    }

    pub fn get_variables(
        &mut self,
        num_variables: i64,
        num_possibilities: i64,
    ) -> (Vec<Variable>, Vec<Variable>) {
        let mut choice_res = Vec::new();
        let mut target_res = Vec::new();

        if num_possibilities <= 0 {
            return (choice_res, target_res);
        }

        let num_variables = min(num_variables, num_possibilities);

        let new_choice_variable_base_name = self.choice_variable_base_name.clone()
            + "_"
            + &self.current_variable_number.to_string();
        let new_target_variable_base_name = self.target_variable_base_name.clone()
            + "_"
            + &self.current_variable_number.to_string();
        self.current_variable_number += 1;

        let mut current_bound = num_possibilities;

        for variable_num in 0..num_variables {
            let new_choice_variable_name =
                new_choice_variable_base_name.clone() + "_" + &variable_num.to_string();
            let new_choice_variable = Variable::Instance(new_choice_variable_name);

            self.variable_set.push(new_choice_variable.clone());
            choice_res.push(new_choice_variable);

            // bound for this variable
            self.variable_bounds.push(current_bound);
            current_bound -= 1;

            let new_target_variable_name =
                new_target_variable_base_name.clone() + "_" + &variable_num.to_string();
            let new_target_variable = Variable::Instance(new_target_variable_name);
            target_res.push(new_target_variable);
        }

        (choice_res, target_res)
    }
}
