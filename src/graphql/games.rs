use super::super::entity::{
    game_entities::{GameEntity, GameInsert},
    puzzle_entities::PuzzleInsert,
};
use super::super::model::{paginated::Paginated, puzzle, user::User, Date};
use super::super::schema;
use super::super::service::pagination::*;
use super::*;
use chrono::{DateTime, Utc};
use diesel::dsl::insert_into;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use juniper::{self, FieldResult};

#[derive(GraphQLObject, Debug)]
pub struct GameDTO {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub available_from: Date,
    pub available_to: Date,
    pub published: bool,
    pub is_owner: bool,
}

pub type PaginatedGames = Paginated<GameDTO>;

graphql_object!(PaginatedGames: () |&self| {
    field items() -> &Vec<GameDTO> {
        &self.items
    }

    field total_pages() -> i32 {
        self.total_pages as i32
    }

    field page() -> i32 {
        self.page as i32
    }
});

#[derive(GraphQLInputObject, Debug)]
pub struct GameSubmissionDTO {
    pub name: String,
    pub words: Vec<String>,
    pub available_from: Option<DateTime<Utc>>,
    pub available_to: Option<DateTime<Utc>>,
}

pub fn fetch_games(
    connection: &DieselConnection,
    current_user: &Option<User>,
    page: Option<i32>,
) -> FieldResult<PaginatedGames> {
    use self::schema::games::dsl::{available_from, available_to, games, owner_id, published};
    use self::schema::users::dsl::users;

    let page = page.unwrap_or(0) as i64;
    let query = games
        .inner_join(users)
        .order_by(available_from.desc())
        .into_boxed();
    let is_avialable_query = published
        .eq(true)
        .and(available_from.le(Utc::now()))
        .and(available_to.gt(Utc::now()).or(available_to.is_null()));
    let query = if let Some(current_user) = &current_user {
        query.filter(owner_id.eq(current_user.id).or(is_avialable_query))
    } else {
        query.filter(is_avialable_query)
    };
    let items = query
        .paginate(page)
        .per_page(25)
        .load_and_count_pages::<(GameEntity, User)>(&connection);
    let (items, total_pages) = items.map_err(|err| {
        error!("Failed to read games {:?}", err);
        "Failed to read games"
    })?;
    let user_id = current_user.as_ref().map(|u| u.id).unwrap_or(-1);
    let result = items
        .into_iter()
        .map(|(game, user)| GameDTO {
            id: game.id,
            name: game.name,
            owner: user.name,
            available_from: game.available_from,
            available_to: game.available_to,
            published: game.published,
            is_owner: user.id == user_id,
        })
        .collect();
    Ok(Paginated::new(result, total_pages, page))
}

pub fn fetch_game_by_id(
    connection: &DieselConnection,
    current_user: &Option<User>,
    id: i32,
) -> FieldResult<GameDTO> {
    use self::schema::games::dsl;
    use self::schema::users::dsl::users;

    let query = dsl::games
        .filter(dsl::id.eq(id))
        .inner_join(users)
        .order_by(dsl::available_from.desc())
        .into_boxed();
    let is_avialable_query = dsl::published
        .eq(true)
        .and(dsl::available_from.le(Utc::now()))
        .and(
            dsl::available_to
                .gt(Utc::now())
                .or(dsl::available_to.is_null()),
        );
    let query = if let Some(current_user) = &current_user {
        query.filter(dsl::owner_id.eq(current_user.id).or(is_avialable_query))
    } else {
        query.filter(is_avialable_query)
    };
    let result = query
        .get_result::<(GameEntity, User)>(&connection.0)
        .optional()
        .map_err(|e| {
            error!("Failed to read game {:?}", e);
            "Failed to read the game"
        })?
        .map(|(game, user)| {
            let user_id = user.id;
            GameDTO {
                id: game.id,
                name: game.name,
                owner: user.name,
                available_from: game.available_from,
                available_to: game.available_to,
                published: game.published,
                is_owner: current_user
                    .as_ref()
                    .map(|u| user_id == u.id)
                    .unwrap_or(false),
            }
        })
        .ok_or("Game not found")?;
    Ok(result)
}

pub fn add_game(
    connection: &DieselConnection,
    current_user: &User,
    game_submission: GameSubmissionDTO,
) -> FieldResult<GameDTO> {
    use self::schema::games::dsl::games;
    use self::schema::puzzles::dsl::puzzles;

    // TODO: check if exists
    let result = connection.transaction::<_, DieselError, _>(|| {
        let result = insert_into(games)
            .values(GameInsert {
                name: game_submission.name,
                available_to: game_submission.available_to,
                available_from: game_submission.available_from,
                published: false,
                owner_id: current_user.id,
            })
            .get_result::<GameEntity>(&connection.0)
            .map(|game| GameDTO {
                id: game.id,
                name: game.name,
                owner: current_user.name.clone(),
                available_from: game.available_from,
                available_to: game.available_to,
                published: game.published,
                is_owner: true,
            })?;

        let puzzle = puzzle::Puzzle::from_words(game_submission.words, 100).map_err(|e| {
            error!("failed to generate puzzle, error: {:?}", e);
            DieselError::RollbackTransaction
        })?;

        let (columns, rows) = puzzle.get_shape();
        insert_into(puzzles)
            .values(PuzzleInsert {
                game_id: result.id,
                game_table: puzzle.get_table().into_iter().collect(),
                table_columns: columns as i32,
                table_rows: rows as i32,
                words: puzzle.get_words(),
                solutions: puzzle
                    .get_solutions()
                    .into_iter()
                    .map(|(v1, v2)| vec![v1.x, v1.y, v2.x, v2.y].into_iter())
                    .flatten()
                    .collect(),
            })
            .execute(&connection.0)?;

        Ok(result)
    })?;

    Ok(result)
}
