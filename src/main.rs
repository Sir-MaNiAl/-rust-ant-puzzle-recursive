use std::cmp::max;
use std::collections::HashMap;
use std::convert::TryFrom;

const X: i64 = 1000;
const Y: i64 = 1000;
const STEPS: u16 = 25;

/// Задача про муравья. Рекурсивный метод подсчета клеток.
///
/// Идея метода состоит в замощении всей области перемещения муравья
/// рекурсивно квадратными фрагментами со стороной равной степени 10,
/// расположенной в положительной четверти координатной плоскости.
///
/// Для области подсчитывается количество возможных перемещений при
/// доступном максимальном расстоянии от начала пути.
/// Так область 10x10 полностью заполняется при максимальном
/// расстояние 18 и более. При этом при преодалении значением расстоянии
/// значений 9n-1 происходит переполнение и рекурсивно заполняются
/// соседние квадраты.
/// При преодолении границы переполнения масштаб увеличивается:
/// рассматривается область 100x100 как область размерности 10,
/// заполненная областями размерности 10, обработываемые на уровне
/// рекурсии ниже. Результаты вычислений кешируются для уменьшения
/// вычислительных затрат.
///
/// Для области 100x100 заполнение происходит уже за 36 и более шагов.
/// Переполнение происходит при значениях 18n-1

static SQUARE: [u8; 19] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];
const THRESHHOLD: u8 = 9;

fn main() {
    let mut result: u128;

    if !validate(X, Y, STEPS) {
        println!(
            "Муравей не может быть в точке с координатами ({}, {})",
            X, Y
        );
        println!("Максимальное значение суммы цифр координат: {}.", STEPS);
        return;
    }

    // Определение разрядов чисел в координатах, недоступных для изменения.
    let (reserved_x_steps, reserved_y_steps) = get_reserved_steps(STEPS, X, Y);

    // Недоступные для изменения разряды ограничивают перемещение муравья.
    let free_steps = STEPS - reserved_x_steps - reserved_y_steps;

    // Кеширование для ускорения рекурсивной обработки
    let mut call_cache: HashMap<(u16, u8), u128> = HashMap::new();
    result = run(free_steps, &mut call_cache);

    if (reserved_x_steps == 0) & (reserved_y_steps != 0) {
        // Муравей может перебраться через полуось Y.
        // Ему доступны симметричные относительно оси точки.
        result *= 2;
        // При этом точки на полуоси были засчитаны дважды. Исправляем.
        result -= get_max_number(free_steps as u32);
    } else if (reserved_x_steps != 0) & (reserved_y_steps == 0) {
        // То же для оси X.
        result *= 2;
        result -= get_max_number(free_steps as u32);
    } else if (reserved_x_steps == 0) & (reserved_y_steps == 0) {
        // Муравью так же доступно начало координат. Бонус x4.
        result *= 4;
        // На всех четырёх полуосях повторы, да и ноль засчитан
        // четыре раза вместо одного. Исправляем.
        result -= 4 * get_max_number(free_steps as u32) + 3;
    }
    println!("{}", result);
}

/// Запуск рекурсивного рассчета.
/// step - максимальная разница в координатах,
/// которую может преодалеть муравей.
fn run(step: u16, cache: &mut HashMap<(u16, u8), u128>) -> u128 {
    let place = step / THRESHHOLD as u16;
    _run_per_digit(step, place as u8, cache)
}

/// Рекурсивный расчет числа доступных ячеек.
/// step - максимальная разница в координатах,
/// которую может преодалеть муравей.
/// place - размерность квадратного фрагмента поля.
/// 10^place * 10^place
fn _run_per_digit(step: u16, place: u8, cache: &mut HashMap<(u16, u8), u128>) -> u128 {
    if let Some(&value) = cache.get(&(step, place)) {
        return value;
    }
    let result;
    let breakpoint: u16 = match place {
        0 => 0,
        _ => 9 * place as u16 - 1,
    };
    if step < breakpoint {
        result = _run_per_digit(step, place - 1, cache);
    } else if step < breakpoint + SQUARE.len() as u16 {
        let weights = SQUARE[..=(step - breakpoint) as usize].iter().rev();
        result = match place {
            0 => weights.map(|&x| x as u128).sum(),
            _ => {
                let place = place - 1;
                weights
                    .enumerate()
                    .map(|(pos, &x)| {
                        x as u128 * _run_per_digit(pos as u16 + breakpoint, place, cache)
                    })
                    .sum()
            }
        };
    } else {
        // step >= breakpoint + SQUARE.len()
        let weights = SQUARE.iter();
        result = match place {
            0 => 100,
            _ => {
                let offset = step - SQUARE.len() as u16 + 1;
                let place = place - 1;
                weights
                    .enumerate()
                    .map(|(pos, &x)| x as u128 * _run_per_digit(pos as u16 + offset, place, cache))
                    .sum()
            }
        };
    }
    cache.insert((step, place), result);
    result
}

/// Проверить может ли муравей находиться на указанной клетке.
fn validate(x: i64, y: i64, steps: u16) -> bool {
    sum_digits(x) + sum_digits(y) <= steps
}

/// Получить сумму цифр числа. Знак не учитывается.
fn sum_digits(num: i64) -> u16 {
    num_to_vec(num).iter().map(|x| *x as u16).sum::<u16>()
}

/// Получить вектор цифр числа в порядке big-endian. Знак не учитывается.
fn num_to_vec(num: i64) -> Vec<u8> {
    let mut digits = Vec::new();
    let mut n: u128 = (num as i128).abs() as u128;
    while n > 9 {
        digits.push(get_last_digit(n));
        n = n / 10;
    }
    digits.push(get_last_digit(n));
    digits
}

/// Получить младший десятичный разряд числа.
fn get_last_digit(num: u128) -> u8 {
    match u8::try_from(num % 10) {
        Ok(n) => n,
        _ => unreachable!(),
    }
}

/// Получить максимальное число, до которого можно добраться по оси
/// от нуля, с суммой цифр чисел не превышающих заданного значения.
fn get_max_number(steps: u32) -> u128 {
    let full_places = steps / 9;
    let last_place_max = (steps) % 9 + 1;
    (0..full_places)
        .map(|place| 9 * 10u128.pow(place as u32))
        .sum::<u128>()
        + last_place_max as u128 * 10u128.pow(full_places)
        - 1
}

/// Вычислить сколько шагов нельзя задействовать из-за барьера между
/// разрядами.
fn get_reserved_steps(steps: u16, x: i64, y: i64) -> (u16, u16) {
    let mut x_digits = num_to_vec(X);
    let mut y_digits = num_to_vec(Y);

    // Приведение векторов к одной длине заполнением старших разрядов
    // незначащими нулями.
    let len = max(x_digits.len(), y_digits.len());
    x_digits.resize(len, 0);
    y_digits.resize(len, 0);

    let mut reserved_x_steps = sum_digits(x);
    let mut reserved_y_steps = sum_digits(y);

    let xy_digits: Vec<(u8, u8)> = x_digits
        .iter()
        .cloned()
        .zip(y_digits.iter().cloned())
        .collect();
    let mut free_steps: u16;
    // Проверка разрядов от младшего к старшему хватает ли шагов
    // чтобы изменить. Когда не хватает, более старшие разряды
    // нет смысла проверять - их не изменить без изменения младших.
    for (pos, &(x_digit, y_digit)) in xy_digits.iter().enumerate() {
        free_steps = steps - reserved_x_steps - reserved_y_steps;
        let breakpoint = match pos {
            0 => 0,
            _ => 9 * pos as u16 - 1,
        };
        if free_steps < breakpoint {
            break;
        }
        reserved_x_steps -= x_digit as u16;
        reserved_y_steps -= y_digit as u16;
    }
    (reserved_x_steps, reserved_y_steps)
}
